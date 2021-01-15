use super::Alert;
use super::Notification;
use super::Probe;
use anyhow::{Context, Result};
use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct HTTP {
    schedule: Option<String>,
    url: String,
    method: String,
    expected_code: u16,
}

impl Probe for HTTP {
    fn observe(&self, alerts: &HashMap<String, Vec<Box<dyn Alert>>>) -> Result<()> {
        log::info!(
            "opening url {} with {} request with expected status code {}",
            self.url,
            self.method,
            self.expected_code
        );

        let func = match &self.method as &str {
            "get" => reqwest::blocking::get,
            _ => anyhow::bail!("unknown request method: {}", self.method),
        };
        let resp = func(&self.url)
            .with_context(|| format!("failed to {} request {}", self.method, self.url))?;
        if resp.status().as_u16() != self.expected_code {
            self.notify(
                alerts,
                Notification {
                    from: "http".to_owned(),
                    check: format!(
                        "http {} request to url {} with expected status code {}",
                        self.method, self.url, self.expected_code
                    ),
                    result: format!(
                        "expected status code is {} and actual code is {}",
                        self.expected_code,
                        resp.status().as_u16()
                    ),
                },
            )?
        }
        Ok(())
    }

    fn local_schedule(&self) -> Option<String> {
        self.schedule.to_owned()
    }
}
