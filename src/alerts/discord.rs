use crate::{alerts::Alert, probes::Notification};
use anyhow::Result;
use async_trait::async_trait;
use lazy_static::lazy_static;
use prometheus::{register_counter_vec, CounterVec};
use serde::Serialize;
use serde_derive::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Discord {
    namepass: Option<Vec<String>>,
    webhook_url: String,
}

lazy_static! {
    static ref RUNS_TOTAL: CounterVec = register_counter_vec!(
        "alert_discord_runs_total",
        "run counter for discord alert plugin",
        &["plugin", "webhook_url"]
    )
    .unwrap();
}

#[async_trait]
impl Alert for Discord {
    fn new(namepass: Vec<&str>) -> Self {
        Discord {
            namepass: Some(namepass.into_iter().map(String::from).collect()),
            ..Default::default()
        }
    }

    fn namepass(&self) -> Option<Vec<String>> {
        self.namepass.clone()
    }

    async fn notify(&self, notif: &Notification) -> Result<()> {
        if !self.should_fire(&notif.name) {
            log::info!("should not fire discord alert for {}", &notif.name);
            return Ok(());
        }
        RUNS_TOTAL
            .with_label_values(&[
                "alert.discord",
                "https://discordapp.com/api/webhooks/[redacted]",
            ])
            .inc();

        log::info!("sending discord alert to webhook url {}", self.webhook_url);
        log::debug!("NOTIFICATION: {:?}", notif);

        let pretext = format!("**TRIGGERED `{}`:** {}", notif.from, notif.title);
        let mut payload = Payload {
            username: "Otto".to_string(),
            content: pretext,
            embeds: vec![],
        };
        match notif.message_entries.as_ref() {
            Some(message_entries) => {
                let entries_length = message_entries.len();
                for (i, entry) in message_entries.iter() {
                    payload.embeds.push(Embed {
                        title: format!("[{} of {}] {}", i + 1, entries_length, entry.title),
                        description: entry.description.chars().take(2048).collect(),
                        color: 15590722,
                    })
                }
            }
            None => payload.embeds.push(Embed {
                title: notif.check.clone(),
                description: notif.message.clone(),
                color: 15590722,
            }),
        }
        let client = reqwest::Client::new();
        let result = client.post(&self.webhook_url).json(&payload).send().await;
        match result {
            Ok(_) => Ok(()),
            Err(err) => anyhow::bail!(
                "failed to post message to discord webhook url {}: {}",
                self.webhook_url,
                err
            ),
        }
    }
}

// discord webhook payload structs

#[derive(Debug, Serialize)]
struct Payload {
    username: String,
    content: String,
    embeds: Vec<Embed>,
}

#[derive(Debug, Serialize)]
struct Embed {
    title: String,
    description: String,
    color: u32,
}
