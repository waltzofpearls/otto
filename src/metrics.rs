use super::config::{Config, Prometheus};
use anyhow::Result;
use hyper::{
    header::CONTENT_TYPE, service::Service, Body, Method, Request, Response, Server, StatusCode,
};
use prometheus::{Encoder, TextEncoder};
use std::{
    future::Future,
    pin::Pin,
    task::{Context as TaskContext, Poll},
};
use tokio::sync::broadcast;

pub fn listen_and_serve(config: &Config, stop_tx: broadcast::Sender<bool>) -> Result<()> {
    if let Some(prometheus) = config.prometheus.clone() {
        let addr = prometheus.listen.parse()?;
        let server = Server::bind(&addr).serve(MakeSvc {
            prometheus: prometheus.clone(),
        });
        let mut stop_rx = stop_tx.subscribe();
        let graceful = server.with_graceful_shutdown(async move {
            if let Err(err) = stop_rx.recv().await {
                log::error!("failed to install graceful shutdown handler: {}", err);
                return;
            }
            log::info!("gracefully shutting down prometheus server...");
        });
        tokio::spawn(async move {
            log::info!("listening on http://{}{}...", addr, prometheus.path);
            if let Err(err) = graceful.await {
                log::error!("prometheus server error: {}", err);
            }
        });
    }
    Ok(())
}

struct Svc {
    prometheus: Prometheus,
}

impl Service<Request<Body>> for Svc {
    type Response = Response<Body>;
    type Error = hyper::Error;
    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut TaskContext) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let method = req.method();
        let path = req.uri().path();
        let res = if method == Method::GET && path == self.prometheus.path {
            let encoder = TextEncoder::new();
            let metric_families = prometheus::gather();
            let mut buffer = vec![];
            encoder.encode(&metric_families, &mut buffer).unwrap();
            let metrics = Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, encoder.format_type())
                .body(Body::from(buffer))
                .unwrap();
            Ok(metrics)
        } else {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        };

        Box::pin(async { res })
    }
}

struct MakeSvc {
    prometheus: Prometheus,
}

impl<T> Service<T> for MakeSvc {
    type Response = Svc;
    type Error = hyper::Error;
    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut TaskContext) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        let prometheus = self.prometheus.clone();
        let fut = async move { Ok(Svc { prometheus }) };
        Box::pin(fut)
    }
}
