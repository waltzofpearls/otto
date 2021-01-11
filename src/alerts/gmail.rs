use super::Alert;
use super::Notification;
use serde_derive::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Gmail {
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_relay: String,
    pub from: String,
    pub to: String,
}

impl Alert for Gmail {
    fn notify(&self, notif: &Notification) {
        println!("ALERT -> Gmail");
        println!("{:?}", notif);
    }
}
