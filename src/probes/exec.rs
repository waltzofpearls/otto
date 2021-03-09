use super::{Alert, Notification, Probe};
use anyhow::Result;
use async_trait::async_trait;
use lazy_static::lazy_static;
use prometheus::{register_counter_vec, register_gauge_vec, CounterVec, GaugeVec};
use serde_derive::Deserialize;
use std::{collections::HashMap, process::Command};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Exec {
    name: Option<String>,
    schedule: Option<String>,
    cmd: String,
    args: Option<Vec<String>>,
}

lazy_static! {
    static ref RUNS_TOTAL: CounterVec = register_counter_vec!(
        "probe_exec_runs_total",
        "run counter for exec probe plugin",
        &["plugin", "cmd"]
    )
    .unwrap();
    static ref TRIGGERED_TOTAL: CounterVec = register_counter_vec!(
        "probe_exec_triggered_total",
        "triggered counter for exec probe plugin",
        &["plugin", "cmd"]
    )
    .unwrap();
    static ref TRIGGERED: GaugeVec = register_gauge_vec!(
        "probe_exec_triggered",
        "Exec probe plugin triggered",
        &["plugin", "cmd"]
    )
    .unwrap();
}

#[async_trait]
impl Probe for Exec {
    fn new() -> Self {
        Exec {
            ..Default::default()
        }
    }

    fn local_schedule(&self) -> Option<String> {
        self.schedule.to_owned()
    }

    async fn observe(&self, alerts: &HashMap<String, Vec<Box<dyn Alert>>>) -> Result<()> {
        log::info!("executing command {:?} with args {:?}", self.cmd, self.args);
        RUNS_TOTAL
            .with_label_values(&["probe.exec", &self.cmd])
            .inc();

        let mut triggered = 0;
        let mut cmd = Command::new(&self.cmd);
        if let Some(args) = &self.args {
            cmd.args(args);
        }
        match cmd.output() {
            Err(err) => {
                log::error!(
                    "failed executing command {} with args {:?}: {}",
                    self.cmd,
                    self.args,
                    err
                );
            }
            Ok(output) => {
                if !output.status.success() {
                    log::warn!(
                        "_TRIGGERED_: command {} with args {:?} got code {}",
                        self.cmd,
                        self.args,
                        output.status,
                    );
                    self.notify(
                        alerts,
                        Notification {
                            from: "exec".to_owned(),
                            name: self.name("exec", self.name.to_owned()),
                            check: format!("command `{}` with args `{:?}`", self.cmd, self.args),
                            title: format!(
                                "`{}` `{:?}` got code {}",
                                self.cmd, self.args, output.status
                            ),
                            message: format!(
                                "{}: {}",
                                output.status,
                                String::from_utf8_lossy(&output.stderr)
                            ),
                            message_html: None,
                        },
                    )
                    .await?;
                    TRIGGERED_TOTAL
                        .with_label_values(&["probe.exec", &self.cmd])
                        .inc();
                    triggered = 1;
                }
            }
        };
        TRIGGERED
            .with_label_values(&["probe.exec", &self.cmd])
            .set(triggered as f64);
        Ok(())
    }
}
