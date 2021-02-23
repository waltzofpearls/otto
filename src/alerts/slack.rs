use super::Alert;
use super::Notification;
use anyhow::Result;
use async_trait::async_trait;
use lazy_static::lazy_static;
use prometheus::{register_counter_vec, CounterVec};
use serde_derive::Deserialize;
use slack_hook::{PayloadBuilder, Slack as SlackHook};

#[derive(Debug, Clone, Deserialize)]
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

        let url: &str = &self.webhook_url;
        let slack = match SlackHook::new(url) {
            Ok(slack) => slack,
            Err(err) => {
                anyhow::bail!("failed to create slack webhook with url {}: {}", url, err)
            }
        };
        let payload = match PayloadBuilder::new()
            .text(format!(
                "--------------------\n*TRIGGERED `{}`:* {}\n--------------------\n{}\n{}",
                notif.from,
                notif.title,
                notif.check,
                notif.message.replace("**", "*")
            ))
            .username("Otto")
            .icon_emoji(":robot_face:")
            .build()
        {
            Ok(payload) => payload,
            Err(err) => anyhow::bail!("failed to build slack webhook payload: {}", err),
        };

        match slack.send(&payload) {
            Ok(_) => Ok(()),
            Err(err) => anyhow::bail!(
                "failed to post message to slack webhook url {}: {}",
                self.webhook_url,
                err
            ),
        }
    }
}
