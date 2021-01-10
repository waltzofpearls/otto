mod alerts;
mod probes;

#[macro_use]
mod macros;

use clap::Clap;
use job_scheduler::{Job, JobScheduler};
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

#[derive(Clap)]
#[clap(version = "1.0", author = "Rollie Ma <rollie@rollie.dev>")]
struct Opts {
    #[clap(short, long, default_value = "otto.toml")]
    config: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    SimpleLogger::new().init()?;

    let opts: Opts = Opts::parse();

    let buffer: String = fs::read_to_string(opts.config)?;
    let config: Config = toml::from_str(&buffer)?;

    let probes = probes::register_from(&config);
    let alerts = alerts::register_from(&config);

    let mut sched = JobScheduler::new();
    let global = config.schedule;

    for (name, plugin) in probes.iter() {
        log::info!("starting probe plugin: {}", name);
        let alerts = &alerts;
        sched.add(Job::new(
            plugin.schedule(&global).parse().unwrap(),
            move || {
                plugin.observe(alerts);
            },
        ));
    }

    loop {
        sched.tick();
        std::thread::sleep(Duration::from_millis(500));
    }
}
