use super::Alert;
use super::Probe;
use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct HTTP {
    schedule: Option<String>,
    https: bool,
    url: String,
}

impl Probe for HTTP {
    fn observe(&self, alerts: &HashMap<String, Box<dyn Alert>>) {
        println!("PROBE -> HTTP");
        self.notify(alerts);
    }
    fn local_schedule(&self) -> Option<String> {
        self.schedule.to_owned()
    }
}
