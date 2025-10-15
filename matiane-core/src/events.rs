use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Focused {
    pub title: String,
    pub id: String,
    pub pid: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum Event {
    Focused(Box<Focused>),
    /// An interval liveness check
    Alive,
    /// swaylock: Screen is now locked or asleep
    Lock,
    /// swaylock: Screen is now unlocked or awake
    Unlock,
    /// swayidle: Went to idle state
    Idle,
    /// swayidle: Back to active state
    Active,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimedEvent {
    pub timestamp: DateTime<Utc>,
    pub event: Event,
}
