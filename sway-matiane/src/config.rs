use matiane_core::xdg;
use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;

const LIVE_INTERVAL: Duration = Duration::from_secs(60);

#[derive(Debug, Deserialize)]
pub struct SwayMatianeConfig {
    pub state_dir: PathBuf,
    pub live_interval: Duration,
}

impl Default for SwayMatianeConfig {
    fn default() -> Self {
        Self {
            state_dir: xdg::data_dir(Some(crate::NAME)),
            live_interval: LIVE_INTERVAL
        }
    }
}
