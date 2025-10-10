use anyhow::{Context, Result};
use chrono::Utc;
use clap::{
    arg,
    builder::{PossibleValuesParser, TypedValueParser},
    command, value_parser,
};
use futures::StreamExt;
use log::{LevelFilter, debug, info, trace, warn};
use matiane_core::events::{Event, TimedEvent};
use matiane_core::log::LoggerBuilder;
use matiane_core::store::EventWriter;
use matiane_core::xdg::Xdg;
use std::path::PathBuf;
use std::str::FromStr;
use sway_matiane::sway::command::EventType;
use sway_matiane::sway::connection::subscribe;
use tokio::time::{MissedTickBehavior, interval};

mod config;

pub const NAME: &str = "mematiane";

#[tokio::main]
async fn main() -> Result<()> {
    let xdg = Xdg::new(NAME.into());

    let ParsedArgs {
        config_file,
        log_level,
    } = parse_args(&xdg);

    init_logger(log_level)?;

    debug!("Loading config");
    let cfg = load_config(&config_file).await?;

    let swaysock_path: PathBuf = std::env::var("SWAYSOCK")
        .with_context(|| "Could not find swaysock env var.")?
        .into();

    let state_dir = cfg.state_dir;
    let now = Utc::now();
    debug!("Opening store");
    let mut write_store = EventWriter::open(state_dir, now).await?;

    debug!("Opening swaysocket");
    let mut events = subscribe(&swaysock_path, EventType::Window).await?;
    let mut alive_interval = interval(cfg.live_interval);
    alive_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    info!("Mematiane has started");

    let mut mematiene_events = events;

    loop {
        tokio::select! {
            event = mematiene_events.next() => {
                match event {
                    Some(event) => {
                        trace!("Received an event");
                        println!("Receved events: {:?}", event);
                    },
                    None => {
                        info!("Sway sock has closed");
                    },
                };
            },

            _ = alive_interval.tick() => {
                let timed = TimedEvent {
                    timestamp: Utc::now(),
                    event: Event::Alive,
                };

                trace!("Alive tick");
                write_store.write(&timed).await?;
            },

            _ = tokio::signal::ctrl_c() => {
                debug!("CTRL-C detected, closing");
                break;
            },
        }
    }

    Ok(())
}

struct ParsedArgs {
    config_file: PathBuf,
    log_level: LevelFilter,
}

fn parse_args(xdg: &Xdg) -> ParsedArgs {
    let possible_levels: Vec<_> =
        LevelFilter::iter().map(|v| v.as_str()).collect();

    let matches = command!("Sway matiane logger")
        .arg(
            arg!(-c --config <FILE> "Sets a custom config file")
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(-l --level <LEVEL> "Sets a log level")
                .value_parser(
                    PossibleValuesParser::new(possible_levels)
                        .map(|s| LevelFilter::from_str(&s).unwrap()),
                )
                .ignore_case(true)
                .default_value("INFO"),
        )
        .get_matches();

    let log_level = *matches.get_one::<LevelFilter>("level").unwrap();
    let config_file = matches
        .get_one::<PathBuf>("config")
        .cloned()
        .unwrap_or_else(|| xdg.config_dir().join("config.toml"));

    ParsedArgs {
        config_file,
        log_level,
    }
}

async fn load_config(file: &PathBuf) -> Result<config::SwayMatianeConfig> {
    let file_str = match tokio::fs::read_to_string(file).await {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(config::SwayMatianeConfig::default());
        }
        Err(e) => return Err(e).context("Failed to read configuration file"),
    };

    let parsed = toml::from_str::<config::SwayMatianeConfig>(&file_str)
        .context("Failed to parse TOML from configuration file")?;

    Ok(parsed)
}

fn init_logger(level: LevelFilter) -> Result<()> {
    let logger = LoggerBuilder::new()
        .with_level(level)
        .to_stderr(true)
        .build();

    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(level);
    Ok(())
}
