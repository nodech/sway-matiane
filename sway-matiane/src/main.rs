#![cfg(target_os = "linux")]
use anyhow::{Context, Result};
use chrono::Utc;
use clap::{
    arg,
    builder::{PossibleValuesParser, TypedValueParser},
    command, value_parser,
};
use futures::{StreamExt, future::ready};
use log::{LevelFilter, debug, error, info, trace, warn};
use matiane_core::events::{Event, Focused, TimedEvent};
use matiane_core::log::init_global_logger;
use matiane_core::process::RunningHandle;
use matiane_core::store::{EventWriter, acquire_lock_file};
use matiane_core::xdg::Xdg;
use std::path::PathBuf;
use std::str::FromStr;
use sway_matiane::{config, sway, swayidle, tray};
use tokio::signal::unix::{SignalKind, signal};
use tokio::time::{MissedTickBehavior, interval};
use tokio_util::sync::CancellationToken;

use sway::{
    command::EventType, connection::subscribe, reply::Event as SwayEvent,
};

#[tokio::main]
async fn main() -> Result<()> {
    let xdg = Xdg::new(matiane_core::NAME.into());

    let ParsedArgs {
        config_file,
        log_level,
    } = parse_args(&xdg);

    init_global_logger(log_level)?;

    debug!("Loading config...");
    let cfg = load_config(&config_file).await?;
    trace!("Config: {:?}", cfg);

    let swaysock_path: PathBuf = std::env::var("SWAYSOCK")
        .with_context(|| "Could not find swaysock env var.")?
        .into();

    let state_dir = cfg.general.state_dir;
    let now = Utc::now();

    debug!("Acquiring lockfile...");
    let lockfile = acquire_lock_file(state_dir.clone()).await?;

    debug!("Opening store...");
    let mut write_store = EventWriter::open(state_dir, now).await?;

    debug!("Running swayidle...");
    info!("Idle timoeut is set to: {} seconds.", cfg.sway.idle_timeout);
    let cancel_tok = CancellationToken::new();
    let sway_idle = run_swayidle(cfg.sway.idle_timeout, cancel_tok.clone())?;

    debug!("Opening swaysocket...");
    let events = subscribe(&swaysock_path, EventType::Window).await?;
    let mut alive_interval = interval(cfg.sway.live_interval);
    alive_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    debug!("Showing tray...");
    let _tray = tray::spawn_tray(cancel_tok.clone());

    info!("Mematiane has started!");

    // Transform sway event into matiane event.
    let mut mematiene_events = events
        .filter(|event| match event {
            Ok(SwayEvent::Window(_)) => ready(true),
            Ok(_) => ready(false),
            Err(err) => {
                warn!("Sway event returned an error {:?}", err);
                ready(false)
            }
        })
        .map(|event| {
            let SwayEvent::Window(mut win_event) = event? else {
                // must not happen, maybe rewrite to return concrete type?
                return Err(anyhow::anyhow!("Incorrect sway event type!"));
            };

            let app_id = win_event.container.app_id.take().or_else(|| {
                let win_props = win_event.container.window_properties.take()?;
                win_props.instance.or(win_props.class)
            });

            let title =
                win_event.container.name.take().or_else(|| app_id.clone());
            let pid = win_event.container.pid.unwrap_or(0);

            let matiane_event = Box::new(Focused {
                title: title.unwrap_or_else(|| "title-not-found".to_string()),
                id: app_id.unwrap_or_else(|| "app-id-not-found".to_string()),
                pid,
            });

            Ok::<Event, anyhow::Error>(Event::Focused(matiane_event))
        });

    let mut sigusr1 = signal(SignalKind::user_defined1())?;
    let mut sigusr2 = signal(SignalKind::user_defined2())?;
    let mut idle = signal(SignalKind::from_raw(libc::SIGRTMIN() + 1))?;
    let mut resume = signal(SignalKind::from_raw(libc::SIGRTMIN() + 2))?;

    loop {
        tokio::select! {
            event = mematiene_events.next() => {
                match event {
                    Some(Ok(event)) => {
                        trace!("Received an event.");
                        write_store.write(&timed_event(event)).await?;
                    }
                    Some(Err(err)) => {
                        error!("Received errored event: {:?}", err);
                        break;
                    },
                    None => {
                        error!("Sway socket has been closed.");
                        break;
                    },
                };
            },

            _ = alive_interval.tick() => {
                trace!("Live tick.");
                write_store.write(&timed_event(Event::Alive)).await?;
            },

            _ = sigusr1.recv() => {
                debug!("Sleeping or locking...");
                write_store.write(&timed_event(Event::Sleep)).await?;
            },

            _ = sigusr2.recv() => {
                debug!("Waking up or unlocking...");
                write_store.write(&timed_event(Event::Awake)).await?;
            },

            _ = idle.recv() => {
                debug!("Idle for {} seconds.", cfg.sway.idle_timeout);
                write_store.write(&timed_event(Event::Idle)).await?;
            },

            _ = resume.recv() => {
                debug!("Resumed.");
                write_store.write(&timed_event(Event::Active)).await?;
            },

            _ = tokio::signal::ctrl_c() => {
                debug!("SIGINT/CTRL-C detected!");
                cancel_tok.cancel();
                break;
            },
        }
    }

    info!("Closing matiane...");
    drop(sway_idle);
    drop(lockfile);

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

async fn load_config(file: &PathBuf) -> Result<config::SwayCliConfig> {
    let file_str = match tokio::fs::read_to_string(file).await {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(config::SwayCliConfig::default());
        }
        Err(e) => return Err(e).context("Failed to read configuration file"),
    };

    let parsed = toml::from_str::<config::SwayCliConfig>(&file_str)
        .context("Failed to parse TOML from configuration file")?;

    Ok(parsed)
}

fn timed_event(event: Event) -> TimedEvent {
    TimedEvent {
        timestamp: Utc::now(),
        event,
    }
}

fn run_swayidle(
    idletimer: u32,
    token: CancellationToken,
) -> Result<RunningHandle> {
    let mut sway_idle = swayidle::SwayIdle::new();
    let pid = std::process::id();

    let idlesignal = libc::SIGRTMIN() + 1;
    let resumesignal = libc::SIGRTMIN() + 2;
    let sigusr1 = libc::SIGUSR1;
    let sigusr2 = libc::SIGUSR2;

    let before_sleep =
        swayidle::BeforeSleep::new(format!("kill -{} {}", sigusr1, pid));
    let after_sleep =
        swayidle::AfterResume::new(format!("kill -{} {}", sigusr2, pid));

    let on_idle = swayidle::Timeout::new_with_resume(
        format!("kill -{} {}", idlesignal, pid),
        idletimer,
        format!("kill -{} {}", resumesignal, pid),
    );

    sway_idle.add_command(before_sleep);
    sway_idle.add_command(after_sleep);
    sway_idle.add_command(on_idle);

    sway_idle.spawn(token)
}
