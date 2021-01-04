use serde_derive::Deserialize;
use std::collections::HashMap;
use super::Config;
use super::plugin_from;

pub mod slack;
pub mod gmail;

#[derive(Debug, Deserialize)]
pub struct Alerts {
    pub slack: Option<slack::Slack>,
    pub gmail: Option<gmail::Gmail>,
}

pub fn register_from(config: &Config) -> HashMap<String, Box<dyn Alert>> {
    let mut alerts = HashMap::new();
    let mut plugin: Box<dyn Alert>;
    match plugin_from!(config.alerts, slack) {
        Some(plg) => { plugin = Box::new(plg.clone()); alerts.insert("slack".to_string(), plugin); println!(""); },
        None => println!(""),
    };
    match plugin_from!(config.alerts, gmail) {
        Some(plg) => { plugin = Box::new(plg.clone()); alerts.insert("gmail".to_string(), plugin); println!(""); },
        None => println!(""),
    };
    alerts
}

pub trait Alert {
    fn notify(&self);
}
