use super::probes::Notification;
use super::register_plugins;
use super::Config;
use serde_derive::Deserialize;
use std::collections::HashMap;

pub mod gmail;
pub mod slack;

#[derive(Debug, Deserialize)]
pub struct Alerts {
    pub slack: Option<Vec<slack::Slack>>,
    pub gmail: Option<Vec<gmail::Gmail>>,
}

pub fn register_from(config: &Config) -> HashMap<String, Vec<Box<dyn Alert>>> {
    let mut alerts = HashMap::new();
    register_plugins!(Alert => config.alerts.slack);
    register_plugins!(Alert => config.alerts.gmail);
    alerts
}

pub trait Alert {
    fn notify(&self, notif: &Notification);
}
