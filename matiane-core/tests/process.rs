#[cfg(target_os = "linux")]
use anyhow::Result;
use matiane_core::process::{AlwaysCommandOptions, run_always_command};
use std::path::PathBuf;
use std::time::Duration;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

async fn sleep_a_ms() {
    tokio::time::sleep(Duration::from_millis(1)).await;
}

async fn is_process_running(pid: u32, name: &str) -> bool {
    let path = PathBuf::from(format!("/proc/{}/comm", pid));
    let read = match tokio::fs::read_to_string(path).await {
        Ok(content) => content,
        Err(_) => return false,
    };

    read.trim() == name
}

fn sleep_options(
    seconds: f32,
    delay: Option<Duration>,
) -> AlwaysCommandOptions {
    AlwaysCommandOptions {
        name: String::from("sleep"),
        args: vec![seconds.to_string()],

        restart_delay: delay.unwrap_or_else(|| Default::default()),
    }
}

#[tokio::test]
async fn process_cancel_task() {
    let command = sleep_options(60.0, None);
    let cancel = CancellationToken::new();

    let running = run_always_command(command, cancel.clone());
    let last_pid;

    {
        sleep_a_ms().await;
        let pid = running.status.lock().await.pid;
        assert!(
            is_process_running(pid, "sleep").await,
            "process \"{}\" is not running.",
            pid
        );
        last_pid = pid;
    }

    cancel.cancel();

    {
        sleep_a_ms().await;
        let pid = running.status.lock().await.pid;
        assert_eq!(pid, 0);
        assert!(
            !is_process_running(last_pid, "sleep").await,
            "process \"{}\" is running.",
            last_pid
        );
    }
}

#[tokio::test]
async fn process_cancel_on_drop() {
    let command = sleep_options(60.0, None);
    let cancel = CancellationToken::new();

    let running = run_always_command(command, cancel.clone());
    let last_pid;

    {
        sleep_a_ms().await;
        let pid = running.status.lock().await.pid;
        assert!(
            is_process_running(pid, "sleep").await,
            "process \"{}\" is not running.",
            pid
        );
        last_pid = pid;
    }

    drop(running);

    {
        sleep_a_ms().await;
        assert!(
            !is_process_running(last_pid, "sleep").await,
            "process \"{}\" is running.",
            last_pid
        );
    }
}

#[tokio::test]
async fn process_restart_killed() -> Result<()> {
    let command = sleep_options(60.0, None);
    let cancel = CancellationToken::new();

    let running = run_always_command(command, cancel);
    let last_pid;

    {
        sleep_a_ms().await;
        last_pid = running.status.lock().await.pid;
        assert!(
            is_process_running(last_pid, "sleep").await,
            "process \"{}\" is not running.",
            last_pid
        );
    }

    // Kill the process
    let excode = Command::new("kill")
        .args(["-9", &last_pid.to_string()])
        .spawn()?
        .wait()
        .await?;

    assert_eq!(excode.code().unwrap(), 0);

    {
        sleep_a_ms().await;
        assert!(
            !is_process_running(last_pid, "sleep").await,
            "process \"{}\" is still running.",
            last_pid
        );

        let pid = running.status.lock().await.pid;
        assert_ne!(pid, last_pid);
        assert!(
            is_process_running(pid, "sleep").await,
            "process \"{}\" is not running.",
            pid
        );
    }

    Ok(())
}

#[tokio::test]
async fn process_restart_done() {
    // 50 ms sleep + 5 ms restart
    let command = sleep_options(0.05, Some(Duration::from_millis(5)));
    let cancel = CancellationToken::new();

    let running = run_always_command(command, cancel);
    let last_pid;

    {
        sleep_a_ms().await;
        last_pid = running.status.lock().await.pid;
        assert_ne!(last_pid, 0);
        assert!(
            is_process_running(last_pid, "sleep").await,
            "process \"{}\" is not running.",
            last_pid
        );
    }

    tokio::time::sleep(Duration::from_millis(150)).await;

    {
        sleep_a_ms().await;
        let pid = running.status.lock().await.pid;
        assert_ne!(pid, 0);
        assert_ne!(pid, last_pid);

        assert!(
            is_process_running(pid, "sleep").await,
            "process \"{}\" is not running.",
            pid
        );
        assert!(
            !is_process_running(last_pid, "sleep").await,
            "process \"{}\" is not running.",
            last_pid
        );
    }
}
