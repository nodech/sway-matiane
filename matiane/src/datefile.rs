use chrono::{DateTime, Utc};
use futures::StreamExt;
use log::debug;
use std::collections::BTreeSet;
use std::path::Path;
use thiserror::Error;
use tokio::fs::read_dir;
use tokio_stream::wrappers::ReadDirStream;

#[derive(Debug, Error)]
pub enum DateFileError {
    #[error("Store IO Error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct DateFile {
    pub file_date: DateTime<Utc>,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

// pub async fn list_files(path: impl AsRef<Path>) -> Result<Vec<DateFile>, DateFileError> {
//     let dirstream = ReadDirStream::new(read_dir(path).await?);

//     let list = dirstream
//         .filter(async |r| r.is_ok())
//         .collect::<Vec<std::fs::DirEntry>>()
//         .await;

//     println!("List: {:?}", list);

//     Ok(list)
// }
