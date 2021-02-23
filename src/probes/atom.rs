use super::Alert;
use super::Notification;
use super::Probe;
use anyhow::{Context, Result};
use async_trait::async_trait;
use atom_syndication::Feed;
use fancy_regex::Regex;
use lazy_static::lazy_static;
use prometheus::{register_counter_vec, CounterVec};
use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct Atom {
    name: Option<String>,
    schedule: Option<String>,
    feed_url: String,
    title_regex: Option<String>,
    content_regex: Option<String>,
}

lazy_static! {
    static ref RUNS_TOTAL: CounterVec = register_counter_vec!(
        "probe_atom_runs_total",
        "run counter for Atom probe plugin",
        &["plugin", "feed_url"]
    )
    .unwrap();
    static ref TRIGGERED_TOTAL: CounterVec = register_counter_vec!(
        "probe_atom_triggered_total",
        "triggered counter for Atom probe plugin",
        &["plugin", "feed_url"]
    )
    .unwrap();
}

#[async_trait]
impl Probe for Atom {
    fn local_schedule(&self) -> Option<String> {
        self.schedule.to_owned()
    }

    async fn observe(&self, alerts: &HashMap<String, Vec<Box<dyn Alert>>>) -> Result<()> {
        log::info!("checking atom feed {}", self.feed_url);
        RUNS_TOTAL
            .with_label_values(&["probe.atom", &self.feed_url])
            .inc();

        let content = reqwest::get(&self.feed_url).await?.bytes().await?;
        let feed = Feed::read_from(&content[..])?;

        if let Some(latest) = feed.entries.first() {
            let title = &latest.title;
            let link = if !latest.links.is_empty() {
                latest.links[0].href()
            } else {
                ""
            };
            let mut found_incident = false;
            if let Some(title_regex) = self.title_regex.to_owned() {
                let re = Regex::new(&title_regex)
                    .with_context(|| format!("failed parsing title_regex {}", title_regex))?;
                found_incident = re
                    .is_match(&title)
                    .with_context(|| format!("failed checking regex match {}", title_regex))?;
            }
            let mut message = format!("Found latest incident from {}.", self.feed_url);
            if let Some(content) = latest.to_owned().content {
                if let Some(value) = content.value {
                    message = value;
                }
            }
            if let Some(content_regex) = self.content_regex.to_owned() {
                let re = Regex::new(&content_regex)
                    .with_context(|| format!("failed parsing content_regex {}", content_regex))?;
                found_incident = re
                    .is_match(&message)
                    .with_context(|| format!("failed checking regex match {}", content_regex))?;
            }
            if found_incident {
                log::warn!(
                    "_TRIGGERED_: found incident from Atom feed {}",
                    self.feed_url
                );
                self.notify(
                    alerts,
                    Notification {
                        from: "atom".to_owned(),
                        name: self.name("atom", self.name.to_owned()),
                        check: format!("Latest incident from Atom feed {}", self.feed_url),
                        title: format!("{} from {}", title, self.feed_url),
                        message: format!("{}\n{}\n{}", title, link, html2md::parse_html(&message)),
                        message_html: Some(format!("{}<br>{}<br>{}", title, link, &message)),
                    },
                )
                .await?;
                TRIGGERED_TOTAL
                    .with_label_values(&["probe.atom", &self.feed_url])
                    .inc();
            }
        }
        Ok(())
    }
}
