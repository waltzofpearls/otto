use super::Alert;
use super::Probe;
use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct Exec {
    pub schedule: Option<String>,
    pub cmd: String,
    pub args: Option<Vec<String>>,
}

impl Probe for Exec {
    fn observe(&self, alerts: &HashMap<String, Box<dyn Alert>>) {
        println!("PROBE -> EXEC");
        self.notify(alerts);
    }
    fn local_schedule(&self) -> Option<String> {
        self.schedule.to_owned()
    }
}
