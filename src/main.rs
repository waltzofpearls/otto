mod alerts;
mod config;
mod probes;
mod web;

#[macro_use]
mod macros;

use anyhow::{Context, Result};
use clap::Clap;
use config::Config;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::str::FromStr;
use tokio::{
    signal::{
        ctrl_c,
        unix::{signal, SignalKind},
    },
    sync::broadcast,
    time::{sleep, Duration},
};

#[derive(Clap)]
#[clap(version = "0.7.0", author = "Rollie Ma <rollie@rollie.dev>")]
struct Opts {
    #[clap(short, long, default_value = "/etc/otto/otto.toml")]
    config: String,
    #[clap(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    SimpleLogger::new()
        .with_level(LevelFilter::from_str(&opts.log_level)?)
        .init()?;

    let mut hangup = signal(SignalKind::hangup())?;
    let (stop_tx, _) = broadcast::channel(1);

    loop {
        let config_file: &str = &opts.config;
        let buffer: String = std::fs::read_to_string(config_file)
            .with_context(|| format!("could not read file `{}`", config_file))?;
        let config: Config = toml::from_str(&buffer)
            .with_context(|| format!("could not parse toml config file `{}`", config_file))?;
        let default_path = String::from("/tmp/otto");
        let path_to_store = config.path_to_store.as_ref().unwrap_or(&default_path);
        let store = sled::open(path_to_store)?;

        web::start(&config, stop_tx.clone())?;

        let probes = probes::register_from(&config);
        let alerts = alerts::register_from(&config);

        probes::start(&config, store.clone(), probes, alerts, stop_tx.clone())?;

        tokio::select! {
            _ = hangup.recv() => {
                log::info!("SIGHUP received, reloading...");
                stop_tx.send(true)?;
                store.flush()?;
                sleep(Duration::from_secs(1)).await;
                continue;
            }
            _ = ctrl_c() => {
                log::info!("SIGINT received, stopping...");
                stop_tx.send(true)?;
                store.flush()?;
                sleep(Duration::from_secs(1)).await;
                break;
            }
        }
    }

    Ok(())
}
