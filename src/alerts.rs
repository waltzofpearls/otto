use super::probes::Notification;
use super::register_plugins;
use super::Config;
use anyhow::Result;
use serde_derive::Deserialize;
use std::collections::HashMap;
use wildmatch::WildMatch;

pub mod email;
pub mod slack;

#[derive(Debug, Deserialize)]
pub struct Alerts {
    pub slack: Option<Vec<slack::Slack>>,
    pub email: Option<Vec<email::Email>>,
}

pub fn register_from(config: &Config) -> HashMap<String, Vec<Box<dyn Alert>>> {
    let mut alerts = HashMap::new();
    register_plugins!(Alert => config.alerts.slack);
    register_plugins!(Alert => config.alerts.email);
    alerts
}

pub trait Alert {
    fn notify(&self, notif: &Notification) -> Result<()>;
    fn namepass(&self) -> Option<Vec<String>>;

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
