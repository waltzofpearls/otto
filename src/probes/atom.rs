use crate::{
    alerts::Alert,
    probes::{MessageEntry, Notification, Probe},
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use atom_syndication::Feed;
use fancy_regex::Regex;
use lazy_static::lazy_static;
use prometheus::{register_counter_vec, register_gauge_vec, CounterVec, GaugeVec};
use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Deserialize)]
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
    static ref TRIGGERED: GaugeVec = register_gauge_vec!(
        "probe_atom_triggered",
        "Atom probe plugin triggered",
        &["plugin", "feed_url"]
    )
    .unwrap();
}

#[async_trait]
impl Probe for Atom {
    fn new() -> Self {
        Atom {
            ..Default::default()
        }
    }

    fn local_schedule(&self) -> Option<String> {
        self.schedule.to_owned()
    }

    async fn observe(&self, alerts: &HashMap<String, Vec<Box<dyn Alert>>>) -> Result<()> {
        log::info!("checking atom feed {}", self.feed_url);
        RUNS_TOTAL
            .with_label_values(&["probe.atom", &self.feed_url])
            .inc();

        let mut triggered = 0;
        let content = reqwest::get(&self.feed_url).await?.bytes().await?;
        let feed = Feed::read_from(&content[..])?;

        let mut found_incidents = 0;
        let mut messages: Vec<String> = vec![];
        let mut messages_html: Vec<String> = vec![];
        let mut message_entries: Vec<(i8, MessageEntry)> = vec![];
        let default_link = atom_syndication::Link::default();

        for entry in feed.entries[0..5].to_vec().iter() {
            let title = &entry.title;
            let link = entry.links.first().unwrap_or(&default_link).href();
            let mut found_incident = false;
            if let Some(title_regex) = self.title_regex.to_owned() {
                let re = Regex::new(&title_regex)
                    .with_context(|| format!("failed parsing title_regex {}", title_regex))?;
                found_incident = re
                    .is_match(&title)
                    .with_context(|| format!("failed checking regex match {}", title_regex))?;
            }
            let mut message = format!("Found incident from {}.", self.feed_url);
            if let Some(content) = entry.to_owned().content {
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
                let message_markdown = html2md::parse_html(&message);
                messages.push(format!("{}\n{}\n{}", title, link, message_markdown));
                messages_html.push(format!("{}<br>{}<br>{}", title, link, &message));
                message_entries.push((
                    found_incidents, // index of this message entry
                    MessageEntry {
                        title: title.to_string(),
                        description: format!("{}\n{}", link, message_markdown),
                    },
                ));
                found_incidents += 1;
            }
        }

        if found_incidents > 0 {
            log::warn!(
                "_TRIGGERED_: found incident from Atom feed {}",
                self.feed_url
            );
            self.notify(
                alerts,
                Notification {
                    from: "atom".to_owned(),
                    name: self.name("atom", self.name.to_owned()),
                    check: format!("Incidents from Atom feed {}", self.feed_url),
                    title: format!(
                        "Found {} incident(s) from {}",
                        found_incidents, self.feed_url
                    ),
                    message: messages.join("\n\n------------------------------\n\n"),
                    message_html: Some(messages_html.join("<br><br><hr><br><br>")),
                    message_entries: Some(message_entries),
                },
            )
            .await?;
            TRIGGERED_TOTAL
                .with_label_values(&["probe.atom", &self.feed_url])
                .inc();
            triggered = 1;
        }

        TRIGGERED
            .with_label_values(&["probe.atom", &self.feed_url])
            .set(triggered as f64);
        Ok(())
    }
}
