use super::Alert;
use super::Notification;
use anyhow::Result;
use serde_derive::Deserialize;
use slack_hook::{PayloadBuilder, Slack as SlackHook};
use std::error::Error;

#[derive(Debug, Clone, Deserialize)]
pub struct Slack {
    pub webhook_url: String,
}

impl Alert for Slack {
    fn notify(&self, notif: &Notification) -> Result<(), Box<dyn Error>> {
        log::info!("sending slack alert to webhook url {}", self.webhook_url);
        log::debug!("NOTIFICATION: {:?}", notif);

        let url: &str = &self.webhook_url;
        let slack = match SlackHook::new(url) {
            Ok(slack) => slack,
            Err(err) => {
                return Err(format!(
                    "failed to create slack webhook with url {}: {}",
                    url, err
                ))?
            }
        };
        let payload = match PayloadBuilder::new()
            .text(format!(
                "Alert received from [*{}*] plugin:\n> {}\n```{}```",
                notif.from, notif.check, notif.result
            ))
            .username("Otto")
            .icon_emoji(":robot_face:")
            .build()
        {
            Ok(payload) => payload,
            Err(err) => return Err(format!("failed to build slack webhook payload: {}", err))?,
        };

        match slack.send(&payload) {
            Ok(_) => Ok(()),
            Err(err) => Err(format!(
                "failed to post message to slack webhook url {}: {}",
                self.webhook_url, err
            ))?,
        }
    }
}
