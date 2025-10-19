use crate::xdg;
use serde::Deserialize;
use std::path::PathBuf;

fn default_state_dir() -> PathBuf {
    xdg::data_dir(Some(crate::NAME))
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GeneralConfig {
    #[serde(default = "default_state_dir")]
    pub state_dir: PathBuf,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        GeneralConfig {
            state_dir: default_state_dir(),
        }
    }
}
