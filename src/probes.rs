use super::alerts::Alert;
use super::register_plugins;
use super::Config;
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use cron::Schedule;
use serde_derive::Deserialize;
use std::{collections::HashMap, str::FromStr, sync::Arc};

pub mod atom;
pub mod exec;
pub mod http;
pub mod rss;

#[derive(Debug, Clone, Deserialize)]
pub struct Probes {
    pub atom: Option<Vec<atom::Atom>>,
    pub exec: Option<Vec<exec::Exec>>,
    pub http: Option<Vec<http::HTTP>>,
    pub rss: Option<Vec<rss::RSS>>,
}

pub fn register_from(config: &Config) -> HashMap<String, Vec<Box<dyn Probe>>> {
    let mut probes = HashMap::new();
    register_plugins!(Probe => config.probes.atom);
    register_plugins!(Probe => config.probes.exec);
    register_plugins!(Probe => config.probes.http);
    register_plugins!(Probe => config.probes.rss);
    probes
}

pub fn start(
    config: &Config,
    probes: HashMap<String, Vec<Box<dyn Probe>>>,
    alerts: HashMap<String, Vec<Box<dyn Alert>>>,
) -> Result<()> {
    // to be shared by concurrent tokio tasks
    let alerts = Arc::new(alerts);
    let global = config.schedule.clone();

    for (name, plugins) in probes.into_iter() {
        log::info!("starting plugins: {} x {}", plugins.len(), name);
        for plugin in plugins.into_iter() {
            let schedule = Schedule::from_str(&plugin.schedule(&global))?;
            let name = name.clone();
            let cloned_alerts = Arc::clone(&alerts);
            tokio::spawn(async move {
                let local_alerts = cloned_alerts;
                for datetime in schedule.upcoming(Utc) {
                    let now = Utc::now();
                    if let Ok(duration) = datetime.signed_duration_since(now).to_std() {
                        tokio::time::sleep(duration).await;
                        plugin
                            .observe(local_alerts.as_ref())
                            .await
                            .unwrap_or_else(|err| {
                                log::error!("[probe][{}] error running plugin: {}", name, err);
                            });
                    }
                }
            });
        }
    }

    Ok(())
}

#[async_trait]
pub trait Probe: Send + Sync {
    fn local_schedule(&self) -> Option<String>;
    async fn observe(&self, alerts: &HashMap<String, Vec<Box<dyn Alert>>>) -> Result<()>;

    fn schedule(&self, global: &str) -> String {
        match self.local_schedule() {
            Some(sched) => sched,
            None => global.to_string(),
        }
    }

    fn name(&self, from: &str, name: Option<String>) -> String {
        format!(
            "{}.{}",
            from,
            match name {
                Some(name) => name,
                None => "".to_owned(),
            }
        )
    }

    fn notify(
        &self,
        alerts: &HashMap<String, Vec<Box<dyn Alert>>>,
        notif: Notification,
    ) -> Result<()> {
        for (name, plugins) in alerts.iter() {
            log::info!(
                "[{}] calling alert plugins: {} x {}",
                notif.from,
                plugins.len(),
                name
            );
            for plugin in plugins.iter() {
                plugin.notify(&notif).unwrap_or_else(|err| {
                    log::error!("[alert][{}] error running plugin: {}", name, err)
                })
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Notification {
    pub from: String,
    pub name: String,
    // command that executed or http url opened
    pub check: String,
    pub title: String,
    // alert message
    pub message: String,
    pub message_html: Option<String>,
}
