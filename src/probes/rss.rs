use super::Alert;
use super::Notification;
use super::Probe;
use anyhow::{Context, Result};
use fancy_regex::Regex;
use rss::Channel;
use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct RSS {
    pub schedule: Option<String>,
    pub feed_url: String,
    pub title_regex: Option<String>,
    pub description_regex: Option<String>,
}

impl Probe for RSS {
    fn observe(&self, alerts: &HashMap<String, Vec<Box<dyn Alert>>>) -> Result<()> {
        log::info!("checking rss feed {}", self.feed_url);

        let content = reqwest::blocking::get(&self.feed_url)?.bytes()?;
        let feed = Channel::read_from(&content[..])?;

        if let Some(latest) = feed.items.first() {
            let title = if let Some(title) = latest.to_owned().title {
                title
            } else {
                String::from("")
            };
            let link = if let Some(link) = latest.to_owned().link {
                link
            } else {
                String::from("")
            };
            let mut found_incident = false;
            if let Some(title_regex) = self.title_regex.to_owned() {
                let re = Regex::new(&title_regex)
                    .with_context(|| format!("failed parsing title_regex {}", title_regex))?;
                found_incident = re
                    .is_match(&title)
                    .with_context(|| format!("failed checking regex match {}", title_regex))?;
            }
            let message = if let Some(description) = latest.to_owned().description {
                description
            } else {
                format!("Found latest incident from {}.", self.feed_url)
            };
            if let Some(description_regex) = self.description_regex.to_owned() {
                let re = Regex::new(&description_regex).with_context(|| {
                    format!("failed parsing content_regex {}", description_regex)
                })?;
                found_incident = re.is_match(&message).with_context(|| {
                    format!("failed checking regex match {}", description_regex)
                })?;
            }
            if found_incident {
                log::info!(
                    "_TRIGGERED_: found incident from RSS feed {}",
                    self.feed_url
                );
                self.notify(
                    alerts,
                    Notification {
                        from: "rss".to_owned(),
                        check: format!("Latest incident from RSS feed {}", self.feed_url),
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
