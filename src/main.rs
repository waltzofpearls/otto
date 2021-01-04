mod alerts;
mod probes;

#[macro_use]
mod macros;

use job_scheduler::{JobScheduler, Job};
use serde_derive::Deserialize;
use simple_logger::SimpleLogger;
use std::error::Error;
use std::fs;
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct Config {
    schedule: String,
    probes: Option<probes::Probes>,
    alerts: Option<alerts::Alerts>,
}

fn main() -> Result<(), Box<dyn Error>> {
    SimpleLogger::new().init()?;

    let buffer: String = fs::read_to_string("./config.toml")?;
    let config: Config = toml::from_str(&buffer)?;

    let probes = probes::register_from(&config);
    let alerts = alerts::register_from(&config);

    let mut sched = JobScheduler::new();
    let global = config.schedule;

    for (name, plugin) in probes.iter() {
        log::info!("starting probe plugin: {}", name);
        let alerts = &alerts;
        sched.add(Job::new(plugin.schedule(&global).parse().unwrap(), move || {
            plugin.observe(alerts);
        }));
    }

    loop {
        sched.tick();
        std::thread::sleep(Duration::from_millis(500));
    }
}
