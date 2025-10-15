use log::debug;
use std::io;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::task::{JoinHandle, spawn};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

pub struct AlwaysCommandOptions {
    pub name: String,
    pub args: Vec<String>,
    pub restart_delay: Duration,
}

impl Default for AlwaysCommandOptions {
    fn default() -> Self {
        AlwaysCommandOptions {
            name: Default::default(),
            args: Default::default(),
            restart_delay: Duration::from_millis(500),
        }
    }
}

#[derive(Default, Debug)]
pub struct RunningStatus {
    pub pid: u32,
}

/// Will kill process on drop.
pub struct RunningHandle {
    pub handle: JoinHandle<Result<(), io::Error>>,
    pub status: Arc<Mutex<RunningStatus>>,
    cancel: CancellationToken,
}

impl Drop for RunningHandle {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

pub fn run_always_command(
    opts: AlwaysCommandOptions,
    token: CancellationToken,
) -> RunningHandle {
    let status = Arc::new(Mutex::new(RunningStatus::default()));
    let spawned_status = status.clone();
    let cancel = token.clone();

    let handle = spawn(async move {
        let AlwaysCommandOptions {
            name,
            args,
            restart_delay,
        } = opts;

        loop {
            if token.is_cancelled() {
                break;
            }

            debug!("Starting command: {}, args: {:?}", &name, &args);
            let mut child = Command::new(&name)
                .args(&args)
                .kill_on_drop(true)
                .stdin(Stdio::null())
                .spawn()?;

            {
                let mut locked = spawned_status.lock().await;
                locked.pid = child.id().unwrap_or(0);
                debug!("Running command pid: {}", locked.pid);
            }

            tokio::select! {
                status = child.wait() => {
                    spawned_status.lock().await.pid = 0;

                    match status {
                        Ok(code) => debug!("Process exitted with {} code.", code),
                        Err(e) => debug!("Process exitted with {} error", e),
                    }

                    debug!("Command is done or was killed, restarting in a sec...");
                    tokio::select! {
                        _ = sleep(restart_delay) => {},
                        _ = token.cancelled() => {
                            break;
                        }
                    }
                },
                _ = token.cancelled() => {
                    break;
                }
            }

            spawned_status.lock().await.pid = 0;
        }

        spawned_status.lock().await.pid = 0;

        Ok(())
    });

    RunningHandle {
        handle,
        status,
        cancel,
    }
}
