use matiane_core::config::GeneralConfig;
use serde::{Deserialize, Deserializer};
use std::time::Duration;

const LIVE_INTERVAL: Duration = Duration::from_secs(60);

fn default_live_interval() -> Duration {
    LIVE_INTERVAL
}

fn default_idle_timeout() -> u32 {
    60
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
    #[serde(
        default = "default_live_interval",
        deserialize_with = "deserialize_interval"
    )]
    pub live_interval: Duration,

    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u32,
}

impl Default for SwayMatianeConfig {
    fn default() -> Self {
        Self {
            live_interval: default_live_interval(),
            idle_timeout: default_idle_timeout(),
        }
    }
}

#[derive(PartialEq, Debug, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct SwayCliConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub sway: SwayMatianeConfig,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn deserialize_config() -> Result<()> {
        #[derive(Debug)]
        struct SuccessCase<'a> {
            raw: &'a str,
            config: SwayCliConfig,
        }

        let tests = vec![
            SuccessCase {
                config: SwayCliConfig {
                    ..Default::default()
                },
                raw: r"",
            },
            SuccessCase {
                config: SwayCliConfig {
                    general: GeneralConfig {
                        state_dir: "/root/state".into(),
                    },
                    ..Default::default()
                },
                raw: r#"
                [general]
                state-dir = "/root/state"
                "#,
            },
            SuccessCase {
                config: SwayCliConfig {
                    sway: SwayMatianeConfig {
                        live_interval: Duration::from_secs(150),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                raw: r#"
                [sway]
                live-interval = 150
                "#,
            },
            SuccessCase {
                config: SwayCliConfig {
                    sway: SwayMatianeConfig {
                        idle_timeout: 150,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                raw: r#"
                [sway]
                idle-timeout = 150
                "#,
            },
            SuccessCase {
                config: SwayCliConfig {
                    general: GeneralConfig {
                        state_dir: "/root/state2".into(),
                    },
                    sway: SwayMatianeConfig {
                        live_interval: Duration::from_secs(20),
                        idle_timeout: 21,
                    },
                },
                raw: r#"
                [general]
                state-dir = "/root/state2"

                [sway]
                live-interval = 20
                idle-timeout = 21
                "#,
            },
        ];

        for test in tests {
            let decoded =
                toml::from_str::<SwayCliConfig>(test.raw).map_err(|err| {
                    anyhow::anyhow!(
                        "Failed: {:?} test case, err: {:?}",
                        test,
                        err
                    )
                })?;

            assert_eq!(decoded, test.config);
        }

        Ok(())
    }
}
