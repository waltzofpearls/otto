use super::alerts::Alerts;
use super::probes::Probes;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub schedule: String,
    pub log_level: String,
    pub probes: Option<Probes>,
    pub alerts: Option<Alerts>,
}
