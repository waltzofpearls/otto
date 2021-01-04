use serde_derive::Deserialize;
use std::collections::HashMap;
use super::Probe;
use super::Alert;

#[derive(Debug, Clone, Deserialize)]
pub struct Exec {
    pub schedule: Option<String>,
    pub cmd: String,
    pub args: Option<Vec<String>>,
}

impl Probe for Exec {
    fn observe(&self, alerts: &HashMap<String, Box<dyn Alert>>) {
        println!("PROBE -> EXEC");
        for (name, plugin) in alerts.into_iter() {
            log::info!("calling alert plugin: {}", name);
            plugin.notify();
        }
    }
    fn local_schedule(&self) -> Option<String> {
        self.schedule.to_owned()
    }
}
