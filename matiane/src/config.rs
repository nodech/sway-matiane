use matiane_core::config::GeneralConfig;
use serde::Deserialize;

#[derive(PartialEq, Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct GuiConfig {}

#[derive(PartialEq, Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct MatianeConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub gui: GuiConfig,
}
