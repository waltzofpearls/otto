use super::alerts::Alert;
use super::plugin_from;
use super::Config;
use serde_derive::Deserialize;
use std::collections::HashMap;

pub mod exec;
pub mod http;

#[derive(Debug, Deserialize)]
pub struct Probes {
    pub exec: Option<exec::Exec>,
    pub http: Option<http::HTTP>,
}

pub fn register_from(config: &Config) -> HashMap<String, Box<dyn Probe>> {
    let mut probes = HashMap::new();
    let mut plugin: Box<dyn Probe>;
    match plugin_from!(config.probes, exec) {
        Some(plg) => {
            plugin = Box::new(plg.clone());
            probes.insert("exec".to_string(), plugin);
        }
        None => println!(""),
    };
    match plugin_from!(config.probes, http) {
        Some(plg) => {
            plugin = Box::new(plg.clone());
            probes.insert("http".to_string(), plugin);
        }
        None => println!(""),
    };
    probes
}

pub trait Probe {
    fn observe(&self, alerts: &HashMap<String, Box<dyn Alert>>);
    fn local_schedule(&self) -> Option<String>;

    fn schedule(&self, global: &str) -> String {
        match self.local_schedule() {
            Some(sched) => sched,
            None => global.to_string(),
        }
    }

    fn notify(&self, alerts: &HashMap<String, Box<dyn Alert>>) {
        for (name, plugin) in alerts.into_iter() {
            log::info!("calling alert plugin: {}", name);
            plugin.notify();
        }
    }
}
