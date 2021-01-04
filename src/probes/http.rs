use serde_derive::Deserialize;
use std::collections::HashMap;
use super::Probe;
use super::Alert;

#[derive(Debug, Clone, Deserialize)]
pub struct HTTP {
    schedule: Option<String>,
    https: bool,
    url: String,
}

impl Probe for HTTP {
    fn observe(&self, alerts: &HashMap<String, Box<dyn Alert>>) {
        println!("PROBE -> HTTP");
        for (name, plugin) in alerts.into_iter() {
            log::info!("calling alert plugin: {}", name);
            plugin.notify();
        }
    }
    fn local_schedule(&self) -> Option<String> {
        self.schedule.to_owned()
    }
}
