use super::Alert;
use super::Notification;
use super::Probe;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone, Deserialize)]
pub struct Exec {
    pub schedule: Option<String>,
    pub cmd: String,
    pub args: Option<Vec<String>>,
}

impl Probe for Exec {
    fn observe(&self, alerts: &HashMap<String, Vec<Box<dyn Alert>>>) {
        log::info!("executing command {:?} with args {:?}", self.cmd, self.args);

        let mut cmd = Command::new(&self.cmd);
        if let Some(args) = &self.args {
            cmd.args(args);
        }
        let output = match cmd.output() {
            Ok(output) => output,
            Err(err) => {
                log::error!("failed to execute command {}", err);
                return;
            }
        };
        if !output.status.success() {
            self.notify(
                alerts,
                Notification {
                    from: "exec".to_owned(),
                    check: format!("command `{}` with args `{:?}`", self.cmd, self.args),
                    result: format!(
                        "{}: {}",
                        output.status,
                        String::from_utf8_lossy(&output.stderr)
                    ),
                },
            );
        }
    }

    fn local_schedule(&self) -> Option<String> {
        self.schedule.to_owned()
    }
}
