use super::Alert;
use super::Notification;
use serde_derive::Deserialize;
use slack_hook::{PayloadBuilder, Slack as SlackHook};

#[derive(Debug, Clone, Deserialize)]
pub struct Slack {
    pub webhook_url: String,
}

impl Alert for Slack {
    fn notify(&self, notif: &Notification) {
        log::info!("ALERT -> SLACK");
        log::debug!("NOTIFICATION: {:?}", notif);

        let url: &str = &self.webhook_url;
        let slack = match SlackHook::new(url) {
            Ok(slack) => slack,
            Err(err) => {
                log::error!("failed to create slack webhook with url {}: {}", url, err);
                return;
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
            Err(err) => {
                log::error!("failed to build slack webhook payload: {}", err);
                return;
            }
        };

        match slack.send(&payload) {
            Ok(()) => return,
            Err(err) => log::error!("failed to post message to slack webhook url: {}", err),
        }
    }
}
