use super::probes::Notification;
use super::register_plugins;
use super::Config;
use anyhow::Result;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::error::Error;

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
    fn notify(&self, notif: &Notification) -> Result<(), Box<dyn Error>>;
}
