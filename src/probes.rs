use super::alerts::Alert;
use super::register_plugins;
use super::Config;
use anyhow::Result;
use serde_derive::Deserialize;
use std::collections::HashMap;

pub mod exec;
pub mod http;

#[derive(Debug, Deserialize)]
pub struct Probes {
    pub exec: Option<Vec<exec::Exec>>,
    pub http: Option<Vec<http::HTTP>>,
}

pub fn register_from(config: &Config) -> HashMap<String, Vec<Box<dyn Probe>>> {
    let mut probes = HashMap::new();
    register_plugins!(Probe => config.probes.exec);
    register_plugins!(Probe => config.probes.http);
    probes
}

pub trait Probe {
    fn observe(&self, alerts: &HashMap<String, Vec<Box<dyn Alert>>>) -> Result<()>;
    fn local_schedule(&self) -> Option<String>;

    fn schedule(&self, global: &str) -> String {
        match self.local_schedule() {
            Some(sched) => sched,
            None => global.to_string(),
        }
    }

    fn notify(
        &self,
        alerts: &HashMap<String, Vec<Box<dyn Alert>>>,
        notif: Notification,
    ) -> Result<()> {
        for (name, plugins) in alerts.into_iter() {
            log::info!("calling alert plugins: {} x {}", plugins.len(), name);
            for plugin in plugins.iter() {
                plugin.notify(&notif).unwrap_or_else(|err| {
                    log::error!("[{}] error running alert plugin: {}", name, err)
                })
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Notification {
    pub from: String,
    // command that executed or http url opened
    pub check: String,
    // alert message
    pub result: String,
}
