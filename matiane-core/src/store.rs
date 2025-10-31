use thiserror::Error;

use super::events::TimedEvent;
use std::time::Duration;
use chrono::{DateTime, NaiveDate, Utc};
use serde_json;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use log::error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("Store IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Store failed to encode event")]
    EncodeError(#[from] serde_json::Error),
}

pub struct EventWriter {
    dir: PathBuf,
    file: File,
    current_date: NaiveDate,
}

impl EventWriter {
    pub async fn open(
        dir: PathBuf,
        date: DateTime<Utc>,
    ) -> Result<Self, StoreError> {
        let filename = get_filename_by_date(date.date_naive());
        let filepath = dir.join(filename);

        let dir_exists = tokio::fs::try_exists(&dir).await?;

        if !dir_exists {
            tokio::fs::create_dir(&dir).await?;
        }

        log::debug!("opening log file: {:?}", filepath);
        let file = open_write_file(filepath).await?;

        let store = EventWriter {
            dir,
            file,
            current_date: date.date_naive(),
        };

        Ok(store)
    }

    pub async fn write(
        &mut self,
        event: &TimedEvent,
    ) -> Result<(), StoreError> {
        self.maybe_rotate(event.timestamp.date_naive()).await?;

        let mut encoded = serde_json::to_vec(&event)?;
        encoded.push(b'\n');

        self.file.write_all(&encoded).await?;

        Ok(())
    }

    pub async fn flush(&mut self) -> Result<(), StoreError> {
        self.file.flush().await.map_err(StoreError::Io)
    }

    pub async fn maybe_rotate(
        &mut self,
        date: NaiveDate,
    ) -> Result<(), StoreError> {
        if self.current_date == date {
            return Ok(());
        }

        let filename = get_filename_by_date(date);
        let filepath = self.dir.join(filename);
        log::debug!("Rotating file: {:?}", filepath);
        let file = open_write_file(filepath).await?;

        self.flush().await?;

        self.file = file;
        self.current_date = date;

        Ok(())
    }
}

async fn open_write_file(filepath: PathBuf) -> Result<File, StoreError> {
    tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(filepath)
        .await
        .map_err(StoreError::Io)
}

pub const LOCK_FILE_TIME_SEC: Duration = Duration::from_secs(60);

#[derive(Debug)]
pub struct LockFile(std::fs::File);

impl Drop for LockFile {
    fn drop(&mut self) {
        if let Err(e) = self.0.unlock() {
            error!("Error unlocking file: {}", e);
        }
    }
}

#[derive(Debug, Error)]
pub enum LockFileError {
    #[error("LockFile IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("LockFile failed to acquire lock.")]
    TryLockError(#[from] std::fs::TryLockError),
    #[error("Opened too soon.")]
    OpenedToSoon,
}

pub async fn acquire_lock_file(filepath: PathBuf) -> Result<LockFile, LockFileError> {
    let filename = filepath.join("LOCK");

    // Time guard ?
    // let stat = tokio::fs::metadata(&filename).await;

    // if let Ok(stat) = stat && let Ok(modified) = stat.modified() {
    //     let diff = std::time::SystemTime::now()
    //         .duration_since(modified)
    //         .unwrap_or(Duration::new(0, 0));

    //     if LOCK_FILE_TIME_SEC.gt(&diff) {
    //         return Err(LockFileError::OpenedToSoon);
    //     }
    // }

    let file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(filename)
        .await
        .map_err(LockFileError::Io)?;

    let stdfile = file.into_std().await;
    stdfile.try_lock()?;
    let lock = LockFile(stdfile);

    Ok(lock)
}

fn get_filename_by_date(date: NaiveDate) -> PathBuf {
    PathBuf::from(date.format("%Y%m%d").to_string()).with_extension("log")
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use chrono::TimeZone;

    #[test]
    fn get_filename_by_name_test() -> Result<()> {
        struct TestCase {
            date: DateTime<Utc>,
            expected: PathBuf,
        }

        let dates = [
            TestCase {
                date: Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap(),
                expected: "19700101.log".into(),
            },
            TestCase {
                date: Utc.with_ymd_and_hms(1970, 1, 1, 23, 59, 59).unwrap(),
                expected: "19700101.log".into(),
            },
            TestCase {
                date: Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap(),
                expected: "20000101.log".into(),
            },
            TestCase {
                date: Utc.with_ymd_and_hms(2000, 1, 1, 5, 6, 7).unwrap(),
                expected: "20000101.log".into(),
            },
            TestCase {
                date: Utc.with_ymd_and_hms(2025, 12, 31, 23, 59, 59).unwrap(),
                expected: "20251231.log".into(),
            },
        ];

        for test in dates {
            let name = get_filename_by_date(test.date.date_naive());
            assert_eq!(name, test.expected);
        }

        Ok(())
    }
}
