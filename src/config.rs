use super::{alerts::Alerts, probes::Probes};
use serde_derive::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub schedule: String,
    pub prometheus: Option<Prometheus>,
    pub probes: Option<Probes>,
    pub alerts: Option<Alerts>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Prometheus {
    pub listen: String,
    pub path: String,
}
