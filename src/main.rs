mod alerts;
mod config;
mod probes;

#[macro_use]
mod macros;

use anyhow::{Context, Result};
use clap::Clap;
use config::Config;
use crossbeam_channel::{bounded, select, tick, Receiver};
use job_scheduler::{Job, JobScheduler};
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::str::FromStr;
use std::time::Duration;

#[derive(Clap)]
#[clap(version = "1.0", author = "Rollie Ma <rollie@rollie.dev>")]
struct Opts {
    #[clap(short, long, default_value = "/etc/otto/otto.toml")]
    config: String,
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    let config_file: &str = &opts.config;
    let buffer: String = std::fs::read_to_string(config_file)
        .with_context(|| format!("could not read file `{}`", config_file))?;
    let config: Config = toml::from_str(&buffer)
        .with_context(|| format!("could not parse toml config file `{}`", config_file))?;

    SimpleLogger::new()
        .with_level(LevelFilter::from_str(&config.log_level)?)
        .init()?;

    let probes = probes::register_from(&config);
    let alerts = alerts::register_from(&config);
    // create metrics registry from probes and alerts
    // and then pass the registry to probe.observe, and then from probes to alerts

    let mut sched = JobScheduler::new();
    let global = config.schedule;

    for (name, plugins) in probes.iter() {
        log::info!("starting probe plugins: {} x {}", plugins.len(), name);
        let alerts = &alerts;
        for plugin in plugins.iter() {
            sched.add(Job::new(
                plugin.schedule(&global).parse().unwrap(),
                move || {
                    plugin.observe(alerts).unwrap_or_else(|err| {
                        log::error!("[probe][{}] error running plugin: {}", name, err);
                    });
                },
            ));
        }
    }

    let ctrl_c_events = ctrl_c_channel()?;
    let ticks = tick(Duration::from_millis(500));

    loop {
        sched.tick();
        select! {
            recv(ticks) -> _ => {}
            recv(ctrl_c_events) -> _ => {
                println!();
                println!("SIGINT received, stopping...");
                break;
            }
        }
    }

    Ok(())
}

fn ctrl_c_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}
