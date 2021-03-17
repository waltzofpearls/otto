use super::{probes::Notification, register_plugins, Config};
use anyhow::Result;
use async_trait::async_trait;
use serde_derive::Deserialize;
use std::collections::HashMap;
use wildmatch::WildMatch;

pub mod discord;
pub mod email;
pub mod slack;
pub mod webhook;

#[derive(Debug, Clone, Deserialize)]
pub struct Alerts {
    pub discord: Option<Vec<discord::Discord>>,
    pub slack: Option<Vec<slack::Slack>>,
    pub email: Option<Vec<email::Email>>,
    pub webhook: Option<Vec<webhook::Webhook>>,
}

pub fn register_from(config: &Config) -> HashMap<String, Vec<Box<dyn Alert>>> {
    let mut alerts = HashMap::new();
    register_plugins!(Alert => config.alerts.discord);
    register_plugins!(Alert => config.alerts.slack);
    register_plugins!(Alert => config.alerts.email);
    register_plugins!(Alert => config.alerts.webhook);
    alerts
}

#[async_trait]
pub trait Alert: Send + Sync {
    fn new(namepass: Vec<&str>) -> Self
    where
        Self: Sized;
    fn namepass(&self) -> Option<Vec<String>>;
    async fn notify(&self, notif: &Notification) -> Result<()>;

    fn should_fire(&self, got: &str) -> bool {
        match self.namepass() {
            // namepass defined: only fire those alerts that match namepass rules
            Some(namepass) => {
                for want in namepass.iter() {
                    if WildMatch::new(want).is_match(got) {
                        return true;
                    }
                }
                false
            }
            // namepass not defined: fire all alerts
            None => true,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test_alert {
        ($test_name:ident, $t:ty) => {
            #[test]
            pub fn $test_name() {
                test_alert_should_fire::<$t>();
            }
        };
    }

    fn test_alert_should_fire<T: Alert>() {
        let plugin: T = T::new(vec!["super-mario", "pokemon-go"]);

        assert_eq!(true, plugin.should_fire("super-mario"));
        assert_eq!(true, plugin.should_fire("pokemon-go"));
        assert_eq!(false, plugin.should_fire("overcooked"));
    }

    test_alert!(test_email_should_fire, email::Email);
    test_alert!(test_slack_should_fire, slack::Slack);
    test_alert!(test_webhook_should_fire, webhook::Webhook);
}
