use super::{alerts::Alert, register_plugins, Config};
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use cron::Schedule;
use serde_derive::{Deserialize, Serialize};
use sled::Db;
use std::{collections::HashMap, str::FromStr, sync::Arc};
use tokio::{sync::broadcast, time::sleep};

pub mod atom;
pub mod exec;
pub mod http;
pub mod rss;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Probes {
    pub atom: Option<Vec<atom::Atom>>,
    pub exec: Option<Vec<exec::Exec>>,
    pub http: Option<Vec<http::Http>>,
    pub rss: Option<Vec<rss::Rss>>,
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
    store: Db,
    probes: HashMap<String, Vec<Box<dyn Probe>>>,
    alerts: HashMap<String, Vec<Box<dyn Alert>>>,
    stop_tx: broadcast::Sender<bool>,
) -> Result<()> {
    // to be shared by concurrent tokio tasks
    let store = Arc::new(store);
    let alerts = Arc::new(alerts);
    let global = config.schedule.clone();

    for (name, plugins) in probes.into_iter() {
        log::info!("starting plugins: {} x {}", plugins.len(), name);
        for plugin in plugins.into_iter() {
            let schedule = Schedule::from_str(&plugin.schedule(&global))?;
            let name = name.clone();
            let cloned_store = Arc::clone(&store);
            let cloned_alerts = Arc::clone(&alerts);
            let mut stop_rx = stop_tx.subscribe();
            tokio::spawn(async move {
                let local_store = cloned_store;
                let local_alerts = cloned_alerts;
                for datetime in schedule.upcoming(Utc) {
                    let now = Utc::now();
                    if let Ok(duration) = datetime.signed_duration_since(now).to_std() {
                        tokio::select! {
                            _ = sleep(duration) => {
                                plugin
                                    .observe(local_store.as_ref(), local_alerts.as_ref())
                                    .await
                                    .unwrap_or_else(|err| {
                                        log::error!("[probe][{}] error running plugin: {}", name, err);
                                    });
                            }
                            _ = stop_rx.recv() => {
                                log::info!("[probe][{}] stopping plugin...", name);
                                break;
                            }
                        } // end select!
                    } // end if
                } // end for
            });
        }
    }

    Ok(())
}

#[async_trait]
pub trait Probe: Send + Sync {
    fn new() -> Self
    where
        Self: Sized;

    fn local_schedule(&self) -> Option<String>;

    fn slug(&self) -> String;

    async fn observe(
        &self,
        store: &sled::Db,
        alerts: &HashMap<String, Vec<Box<dyn Alert>>>,
    ) -> Result<()>;

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

    async fn notify(
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
                plugin.notify(&notif).await.unwrap_or_else(|err| {
                    log::error!("[alert][{}] error running plugin: {}", name, err)
                })
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default, Serialize)]
pub struct Notification {
    pub from: String,
    pub name: String,
    // command executed, http url opened, atom or rss feed parsed
    pub check: String,
    pub title: String,
    // alert message
    pub message: String,
    pub message_html: Option<String>,
    pub message_entries: Option<Vec<(i8, MessageEntry)>>,
}

#[derive(Debug, Default, Serialize)]
pub struct MessageEntry {
    pub title: String,
    pub description: String,
}

const HAS_INCIDENT: &[u8] = &[1, 1, 1];
const NO_INCIDENT: &[u8] = &[0, 0, 0];

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test_probe {
        ($test_name:ident, $t:ty) => {
            #[tokio::test]
            async fn $test_name() {
                test_probe_notify::<$t>().await;
            }
        };
    }

    async fn test_probe_notify<T: Probe>() {
        let plugin: T = T::new();
        let mock_alert = MockAlert::new(vec![""]);
        let alerts_vec: Vec<Box<dyn Alert>> = vec![Box::new(mock_alert)];
        let mut alerts_map = HashMap::new();
        alerts_map.insert(String::from("mock_alert"), alerts_vec);
        let result = plugin
            .notify(
                &alerts_map,
                Notification {
                    ..Default::default()
                },
            )
            .await;

        assert_eq!(true, result.is_ok())
    }

    struct MockAlert {}

    #[async_trait]
    impl Alert for MockAlert {
        fn new(_namepass: Vec<&str>) -> Self {
            MockAlert {}
        }
        fn namepass(&self) -> Option<Vec<String>> {
            None
        }
        async fn notify(&self, _notif: &Notification) -> Result<()> {
            Ok(())
        }
    }

    test_probe!(test_atom_notify, atom::Atom);
    test_probe!(test_exec_notify, exec::Exec);
    test_probe!(test_http_notify, http::Http);
    test_probe!(test_rss_notify, self::rss::Rss);
}
