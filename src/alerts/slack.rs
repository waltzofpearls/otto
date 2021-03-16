use super::{Alert, Notification};
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

        let message = notif.message.replace("**", "*");
        let fallback = format!(
            "--------------------\n*TRIGGERED `{}`:* {}\n--------------------\n{}\n{}",
            notif.from, notif.title, notif.check, message
        );
        let title = format!("*TRIGGERED `{}`:* {}", notif.from, notif.title);
        let payload = Payload {
            username: "Otto".into(),
            icon_emoji: ":robot_face:".into(),
            attachments: vec![Attachment {
                fallback,
                pretext: title,
                title: notif.check.clone(),
                text: message,
                color: "#ede542".into(),
                fields: vec![Field {
                    title: "Name".into(),
                    value: notif.name.clone(),
                    short: false,
                }],
            }],
        };
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

#[derive(Debug, Serialize)]
struct Payload {
    username: String,
    icon_emoji: String,
    attachments: Vec<Attachment>,
}

#[derive(Debug, Serialize)]
struct Attachment {
    fallback: String,
    pretext: String,
    text: String,
    color: String,
    fields: Vec<Field>,
    title: String,
}

#[derive(Debug, Serialize)]
pub struct Field {
    title: String,
    value: String,
    short: bool,
}
