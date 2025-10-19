use matiane_core::xdg;
use serde::{Deserialize, Deserializer};
use std::path::PathBuf;
use std::time::Duration;

const LIVE_INTERVAL: Duration = Duration::from_secs(60);

fn default_state_dir() -> PathBuf {
    xdg::data_dir(Some(crate::NAME))
}

fn default_live_interval() -> Duration {
    LIVE_INTERVAL
}

fn deserialize_interval<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let secs = u64::deserialize(deserializer)?;
    Ok(Duration::from_secs(secs))
}

#[derive(PartialEq, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SwayMatianeConfig {
    #[serde(default = "default_state_dir")]
    pub state_dir: PathBuf,
    #[serde(
        default = "default_live_interval",
        deserialize_with = "deserialize_interval"
    )]
    pub live_interval: Duration,
}

impl Default for SwayMatianeConfig {
    fn default() -> Self {
        Self {
            state_dir: default_state_dir(),
            live_interval: default_live_interval(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn deserialize_config() -> Result<()> {
        struct SuccessCase<'a> {
            raw: &'a str,
            config: SwayMatianeConfig,
        }

        let tests = vec![
            SuccessCase {
                config: SwayMatianeConfig {
                    ..Default::default()
                },
                raw: r"",
            },
            SuccessCase {
                config: SwayMatianeConfig {
                    state_dir: "/root/state".into(),
                    ..Default::default()
                },
                raw: r#"
                state-dir = "/root/state"
                "#,
            },
            SuccessCase {
                config: SwayMatianeConfig {
                    live_interval: Duration::from_secs(150),
                    ..Default::default()
                },
                raw: r#"
                live-interval = 150
                "#,
            },
            SuccessCase {
                config: SwayMatianeConfig {
                    state_dir: "/root/state2".into(),
                    live_interval: Duration::from_secs(20),
                },
                raw: r#"
                state-dir = "/root/state2"
                live-interval = 20
                "#,
            },
        ];

        for test in tests {
            let decoded = toml::from_str::<SwayMatianeConfig>(test.raw)?;

            assert_eq!(decoded, test.config);
        }

        Ok(())
    }
}
