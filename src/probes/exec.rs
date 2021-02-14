use super::Alert;
use super::Notification;
use super::Probe;
use anyhow::{Context, Result};
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone, Deserialize)]
pub struct Exec {
    name: Option<String>,
    schedule: Option<String>,
    tags: Option<HashMap<String, String>>,
    cmd: String,
    args: Option<Vec<String>>,
}

impl Probe for Exec {
    fn observe(&self, alerts: &HashMap<String, Vec<Box<dyn Alert>>>) -> Result<()> {
        log::info!("executing command {:?} with args {:?}", self.cmd, self.args);

        let mut cmd = Command::new(&self.cmd);
        if let Some(args) = &self.args {
            cmd.args(args);
        }
        let output = cmd.output().with_context(|| "failed to execute command")?;
        if !output.status.success() {
            log::warn!(
                "_TRIGGERED_: failed executing command {} with args {:?}",
                self.cmd,
                self.args,
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
            )?
        }
        Ok(())
    }

    fn local_schedule(&self) -> Option<String> {
        self.schedule.to_owned()
    }
}
