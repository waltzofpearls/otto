use super::Alert;
use super::Notification;
use super::Probe;
use anyhow::{Context, Result};
use atom_syndication::Feed;
use fancy_regex::Regex;
use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct Atom {
    pub schedule: Option<String>,
    pub feed_url: String,
    pub title_regex: Option<String>,
    pub content_regex: Option<String>,
}

impl Probe for Atom {
    fn observe(&self, alerts: &HashMap<String, Vec<Box<dyn Alert>>>) -> Result<()> {
        log::info!("checking atom feed {}", self.feed_url);

        let content = reqwest::blocking::get(&self.feed_url)?.bytes()?;
        let feed = Feed::read_from(&content[..])?;

        if let Some(latest) = feed.entries.first() {
            let title = &latest.title;
            let link = if latest.links.len() > 0 {
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
                log::info!(
                    "_TRIGGERED_: found incident from Atom feed {}",
                    self.feed_url
                );
                self.notify(
                    alerts,
                    Notification {
                        from: "atom".to_owned(),
                        check: format!("Latest incident from Atom feed {}", self.feed_url),
                        message: format!("{}\n{}\n{}", title, link, html2md::parse_html(&message)),
                        message_html: Some(format!("{}<br>{}<br>{}", title, link, &message)),
                    },
                )?;
            }
        }
        Ok(())
    }

    fn local_schedule(&self) -> Option<String> {
        self.schedule.to_owned()
    }
}
