use super::config::Config;
use anyhow::Result;
use prometheus::{Encoder, TextEncoder};
use tokio::sync::broadcast;
use warp::{Filter, Rejection, Reply};

pub fn start(config: &Config, stop_tx: broadcast::Sender<bool>) -> Result<()> {
    if let Some(prometheus) = config.prometheus.clone() {
        let addr: std::net::SocketAddr = prometheus.listen.parse()?;
        let metrics_route = warp::path(prometheus.path.clone()).and_then(metrics_handler);
        let mut stop_rx = stop_tx.subscribe();
        let (_, server) =
            warp::serve(metrics_route).bind_with_graceful_shutdown(addr, async move {
                if let Err(err) = stop_rx.recv().await {
                    log::error!("failed to install graceful shutdown handler: {}", err);
                    return;
                }
                log::info!("gracefully shutting down prometheus server...");
            });
        log::info!("listening on http://{}/{}...", addr, prometheus.path);
        tokio::spawn(server);
    }
    Ok(())
}

async fn metrics_handler() -> Result<impl Reply, Rejection> {
    let encoder = TextEncoder::new();
    let mut buffer = vec![];
    if let Err(err) = encoder.encode(&prometheus::gather(), &mut buffer) {
        log::error!("failed encoding prometheus metrics: {}", err);
        warp::reject::reject();
    };
    match String::from_utf8(buffer) {
        Ok(res) => Ok(res),
        Err(err) => {
            log::error!("failed converting prometheus metrics from_utf8: {}", err);
            Err(warp::reject::reject())
        }
    }
}
