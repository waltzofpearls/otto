use super::Alert;
use super::Notification;
use anyhow::Result;
use serde_derive::Deserialize;
use slack_hook::{PayloadBuilder, Slack as SlackHook};

#[derive(Debug, Clone, Deserialize)]
pub struct Slack {
    namepass: Option<Vec<String>>,
    webhook_url: String,
}

impl Alert for Slack {
    fn namepass(&self) -> Option<Vec<String>> {
        self.namepass.clone()
    }

    fn notify(&self, notif: &Notification) -> Result<()> {
        if !self.should_fire(&notif.name) {
            log::info!("should not fire slack alert for {}", &notif.name);
            return Ok(());
        }

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
