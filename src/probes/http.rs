use crate::{
    alerts::Alert,
    probes::{Notification, Probe, HAS_INCIDENT, NO_INCIDENT},
};
use anyhow::Result;
use async_trait::async_trait;
use lazy_static::lazy_static;
use prometheus::{register_counter_vec, register_gauge_vec, CounterVec, GaugeVec};
use serde_derive::Deserialize;
use sled::{Db, IVec};
use slug::slugify;
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Http {
    name: Option<String>,
    schedule: Option<String>,
    url: String,
    method: String,
    headers: Option<HashMap<String, String>>,
    json: Option<String>,
    expected_code: u16,
}

lazy_static! {
    static ref RUNS_TOTAL: CounterVec = register_counter_vec!(
        "probe_http_runs_total",
        "run counter for HTTP probe plugin",
        &["plugin", "url", "method"]
    )
    .unwrap();
    static ref TRIGGERED_TOTAL: CounterVec = register_counter_vec!(
        "probe_http_triggered_total",
        "triggered counter for HTTP probe plugin",
        &["plugin", "url", "method"]
    )
    .unwrap();
    static ref TRIGGERED: GaugeVec = register_gauge_vec!(
        "probe_http_triggered",
        "HTTP probe plugin triggered",
        &["plugin", "url", "method"]
    )
    .unwrap();
}

#[async_trait]
impl Probe for Http {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn local_schedule(&self) -> Option<String> {
        self.schedule.to_owned()
    }

    fn slug(&self) -> String {
        slugify(format!(
            "http-{}-{}-{}",
            self.url, self.method, self.expected_code
        ))
    }

    async fn observe(
        &self,
        store: &Db,
        alerts: &HashMap<String, Vec<Box<dyn Alert>>>,
    ) -> Result<()> {
        log::info!(
            "sending [{}] request to {} with expected status code {}",
            self.method,
            self.url,
            self.expected_code
        );
        RUNS_TOTAL
            .with_label_values(&["probe.http", &self.url, &self.method])
            .inc();

        let stored = store.get(self.slug().as_bytes())?;
        let mut to_store = NO_INCIDENT;
        let mut triggered = 0;
        let client = reqwest::Client::new();
        let mut req = match &self.method as &str {
            "get" => client.get(&self.url),
            "post" => client.post(&self.url),
            _ => anyhow::bail!("unknown request method: {}", self.method),
        };
        for (header, value) in self.headers.as_ref().unwrap_or(&HashMap::new()).iter() {
            req = req.header(header, value);
        }
        if let Some(json) = self.json.to_owned() {
            req = req.json(&json)
        }
        let resp = req.send().await;
        let mut title: String = "".to_owned();
        let mut message: String = "".to_owned();
        let mut found_incident = false;
        match resp {
            Ok(resp) => {
                if resp.status().as_u16() != self.expected_code {
                    found_incident = true;
                    title = format!(
                        "{} {} want {} got {}",
                        self.method,
                        self.url,
                        self.expected_code,
                        resp.status().as_u16()
                    );
                    message = format!(
                        "expected status code is {} and actual code is {}",
                        self.expected_code,
                        resp.status().as_u16()
                    );
                }
            }
            Err(err) => {
                found_incident = true;
                if err.is_connect() || err.is_request() || err.is_redirect() || err.is_timeout() {
                    title = format!(
                        "{} {} want {} got error {}",
                        self.method, self.url, self.expected_code, err
                    );
                    message = format!(
                        "expected status code is {} and got error {}",
                        self.expected_code, err
                    );
                } else {
                    anyhow::bail!("failed to {} request {}: {}", self.method, self.url, err);
                }
            }
        }
        if found_incident {
            log::info!(
                "_TRIGGERED_: {} {} with expected code {}",
                self.method,
                self.url,
                self.expected_code,
            );
            if stored.is_none() || stored == Some(IVec::from(NO_INCIDENT)) {
                log::warn!(
                    "_NOTIFY_: {} {} with expected code {}",
                    self.method,
                    self.url,
                    self.expected_code
                );
                self.notify(
                    alerts,
                    Notification {
                        from: "http".to_owned(),
                        name: self.name("http", self.name.to_owned()),
                        check: format!(
                            "http {} request to url {} with expected status code {}",
                            self.method, self.url, self.expected_code
                        ),
                        title,
                        message,
                        message_html: None,
                        message_entries: None,
                    },
                )
                .await?;
            }
            TRIGGERED_TOTAL
                .with_label_values(&["probe.http", &self.url, &self.method])
                .inc();
            triggered = 1;
            to_store = HAS_INCIDENT;
        }

        store.insert(self.slug().as_bytes(), to_store)?;

        TRIGGERED
            .with_label_values(&["probe.http", &self.url, &self.method])
            .set(triggered as f64);
        Ok(())
    }
}
