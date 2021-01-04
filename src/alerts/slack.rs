use super::Alert;
use serde_derive::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Slack {
    pub api_key: String,
}

impl Alert for Slack {
    fn notify(&self) {
        println!("ALERT -> SLACK");
    }
}
