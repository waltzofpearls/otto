use super::Alert;
use super::Notification;
use super::Probe;
use anyhow::{Context, Result};
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

impl Probe for HTTP {
    fn observe(&self, alerts: &HashMap<String, Vec<Box<dyn Alert>>>) -> Result<()> {
        log::info!(
            "sending [{}] request to {} with expected status code {}",
            self.method,
            self.url,
            self.expected_code
        );

        let client = reqwest::blocking::Client::new();
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
                        "{} {} got {} want {}",
                        self.method,
                        self.url,
                        resp.status().as_u16(),
                        self.expected_code
                    ),
                    message: format!(
                        "expected status code is {} and actual code is {}",
                        self.expected_code,
                        resp.status().as_u16()
                    ),
                    message_html: None,
                },
            )?
        }
        Ok(())
    }

    fn local_schedule(&self) -> Option<String> {
        self.schedule.to_owned()
    }
}
