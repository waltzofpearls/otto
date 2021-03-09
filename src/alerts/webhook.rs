use super::{Alert, Notification};
use anyhow::{Context, Result};
use async_trait::async_trait;
use lazy_static::lazy_static;
use prometheus::{register_counter_vec, CounterVec};
use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Webhook {
    namepass: Option<Vec<String>>,
    url: String,
    headers: Option<HashMap<String, String>>,
}

lazy_static! {
    static ref RUNS_TOTAL: CounterVec = register_counter_vec!(
        "alert_webhook_runs_total",
        "run counter for webhook alert plugin",
        &["plugin", "url"]
    )
    .unwrap();
}

#[async_trait]
impl Alert for Webhook {
    fn new(namepass: Vec<&str>) -> Self {
        Webhook {
            namepass: Some(namepass.into_iter().map(String::from).collect()),
            ..Default::default()
        }
    }

    fn namepass(&self) -> Option<Vec<String>> {
        self.namepass.clone()
    }

    async fn notify(&self, notif: &Notification) -> Result<()> {
        if !self.should_fire(&notif.name) {
            log::info!("should not fire webhook alert for {}", &notif.name);
            return Ok(());
        }
        RUNS_TOTAL
            .with_label_values(&["alert.webhook", &self.url])
            .inc();

        log::info!("sending webhook alert to {}", self.url);
        log::debug!("NOTIFICATION: {:?}", notif);

        let client = reqwest::Client::new();
        let mut req = client.post(&self.url);
        for (header, value) in self.headers.as_ref().unwrap_or(&HashMap::new()).iter() {
            req = req.header(header, value);
        }
        let json = serde_json::to_string(notif)?;
        let resp = req
            .json(&json)
            .send()
            .await
            .with_context(|| format!("failed to send webhook alert to {}", self.url))?;

        if resp.status().as_u16() != 200 {
            anyhow::bail!(
                "failed sending webhook alert to {}, expected status code 200, got {}",
                self.url,
                resp.status().as_u16()
            );
        }
        Ok(())
    }
}
