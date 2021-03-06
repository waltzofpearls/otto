use crate::{alerts::Alert, probes::Notification};
use anyhow::Result;
use async_trait::async_trait;
use lazy_static::lazy_static;
use prometheus::{register_counter_vec, CounterVec};
use serde::Serialize;
use serde_derive::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Slack {
    namepass: Option<Vec<String>>,
    webhook_url: String,
}

lazy_static! {
    static ref RUNS_TOTAL: CounterVec = register_counter_vec!(
        "alert_slack_runs_total",
        "run counter for slack alert plugin",
        &["plugin", "webhook_url"]
    )
    .unwrap();
}

#[async_trait]
impl Alert for Slack {
    fn new(namepass: Vec<&str>) -> Self {
        Slack {
            namepass: Some(namepass.into_iter().map(String::from).collect()),
            ..Default::default()
        }
    }

    fn namepass(&self) -> Option<Vec<String>> {
        self.namepass.clone()
    }

    async fn notify(&self, notif: &Notification) -> Result<()> {
        if !self.should_fire(&notif.name) {
            log::info!("should not fire slack alert for {}", &notif.name);
            return Ok(());
        }
        RUNS_TOTAL
            .with_label_values(&["alert.slack", "https://hooks.slack.com/services/[redacted]"])
            .inc();

        log::info!("sending slack alert to webhook url {}", self.webhook_url);
        log::debug!("NOTIFICATION: {:?}", notif);

        let pretext = format!("*TRIGGERED `{}`:* {}", notif.from, notif.title);
        let mut payload = Payload {
            username: "Otto".to_string(),
            icon_emoji: ":robot_face:".to_string(),
            text: pretext.clone(),
            attachments: vec![],
        };
        match notif.message_entries.as_ref() {
            Some(message_entries) => {
                let entries_length = message_entries.len();
                for (i, entry) in message_entries.iter() {
                    payload.attachments.push(Attachment {
                        title: format!("[{} of {}] {}", i + 1, entries_length, entry.title),
                        text: entry.description.replace("**", "*"),
                        color: "#ede542".to_string(),
                    })
                }
            }
            None => payload.attachments.push(Attachment {
                title: notif.check.clone(),
                text: notif.message.replace("**", "*"),
                color: "#ede542".to_string(),
            }),
        }
        let client = reqwest::Client::new();
        let result = client.post(&self.webhook_url).json(&payload).send().await;
        match result {
            Ok(_) => Ok(()),
            Err(err) => anyhow::bail!(
                "failed to post message to slack webhook url {}: {}",
                self.webhook_url,
                err
            ),
        }
    }
}

// slack webhook payload structs

#[derive(Debug, Serialize)]
struct Payload {
    username: String,
    icon_emoji: String,
    text: String,
    attachments: Vec<Attachment>,
}

#[derive(Debug, Serialize)]
struct Attachment {
    title: String,
    text: String,
    color: String,
}
