use super::Alert;
use super::Notification;
use anyhow::Result;
use serde_derive::Deserialize;
use slack_hook::{PayloadBuilder, Slack as SlackHook};

#[derive(Debug, Clone, Deserialize)]
pub struct Slack {
    pub webhook_url: String,
}

impl Alert for Slack {
    fn notify(&self, notif: &Notification) -> Result<()> {
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
                "Alert received from [*{}*] plugin:\n> {}\n```{}```",
                notif.from, notif.check, notif.message
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
