use super::Alert;
use super::Notification;
use super::Probe;
use anyhow::{Context, Result};
use async_trait::async_trait;
use lazy_static::lazy_static;
use prometheus::{register_counter_vec, CounterVec};
use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct HTTP {
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
}

#[async_trait]
impl Probe for HTTP {
    fn local_schedule(&self) -> Option<String> {
        self.schedule.to_owned()
    }

    async fn observe(&self, alerts: &HashMap<String, Vec<Box<dyn Alert>>>) -> Result<()> {
        log::info!(
            "sending [{}] request to {} with expected status code {}",
            self.method,
            self.url,
            self.expected_code
        );
        RUNS_TOTAL
            .with_label_values(&["probe.http", &self.url, &self.method])
            .inc();

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
        let resp = req
            .send()
            .await
            .with_context(|| format!("failed to {} request {}", self.method, self.url))?;
        if resp.status().as_u16() != self.expected_code {
            log::warn!(
                "_TRIGGERED_: failed [{}] requesting url {} with expected code {}",
                self.method,
                self.url,
                self.expected_code,
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
                    title: format!(
                        "{} {} want {} got {}",
                        self.method,
                        self.url,
                        self.expected_code,
                        resp.status().as_u16()
                    ),
                    message: format!(
                        "expected status code is {} and actual code is {}",
                        self.expected_code,
                        resp.status().as_u16()
                    ),
                    message_html: None,
                },
            )
            .await?;
            TRIGGERED_TOTAL
                .with_label_values(&["probe.http", &self.url, &self.method])
                .inc();
        }
        Ok(())
    }
}
