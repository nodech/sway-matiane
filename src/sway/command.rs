// This is based on https://github.com/JayceFayne/swayipc-rs
// https://man.archlinux.org/man/sway-ipc.7#MESSAGES_AND_REPLIES

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommandTypeError {
    #[error("Wrong command type `{0}`.")]
    IncorrectCommandType(u32),
    #[error("Wrong event type `{0}`.")]
    IncorrectEventType(u32),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CommandType {
    /// Runs the payload as sway commands.
    RunCommand = 0,
    /// Get the list of current workspaces.
    GetWorkspaces = 1,
    /// Subscribe the IPC connection to the events listed in the payload.
    Subscribe = 2,
    /// Get the list of current outputs.
    GetOutputs = 3,
    /// Get the node layout tree.
    GetTree = 4,
    /// Get the names of all the marks currently set.
    GetMarks = 5,
    /// Get the specified bar config or a list of bar config names.
    GetBarConfig = 6,
    /// Get the version of sway that owns the IPC socket.
    GetVersion = 7,
    /// Get the list of binding mode names.
    GetBindingModes = 8,
    /// Returns the config that was last loaded.
    GetConfig = 9,
    /// Sends a tick event with the specified payload.
    SendTick = 10,
    /// Replies failure object for i3 compatibility.
    Sync = 11,
    /// Request the current binding state, e.g. the currently active binding
    /// mode name.
    GetBindingState = 12,
    /// Get the list of input devices.
    GetInputs = 100,
    /// Get the list of seats.
    GetSeats = 101,
}

impl TryFrom<u32> for CommandType {
    type Error = CommandTypeError;

    fn try_from(n: u32) -> Result<CommandType, Self::Error> {
        match n {
            0 => Ok(CommandType::RunCommand),
            1 => Ok(CommandType::GetWorkspaces),
            2 => Ok(CommandType::Subscribe),
            3 => Ok(CommandType::GetOutputs),
            4 => Ok(CommandType::GetTree),
            5 => Ok(CommandType::GetMarks),
            6 => Ok(CommandType::GetBarConfig),
            7 => Ok(CommandType::GetVersion),
            8 => Ok(CommandType::GetBindingModes),
            9 => Ok(CommandType::GetConfig),
            10 => Ok(CommandType::SendTick),
            11 => Ok(CommandType::Sync),
            12 => Ok(CommandType::GetBindingState),
            100 => Ok(CommandType::GetInputs),
            101 => Ok(CommandType::GetSeats),
            v => Err(CommandTypeError::IncorrectCommandType(v)),
        }
    }
}

/// Source: https://man.archlinux.org/man/sway-ipc.7#EVENTS
#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// Sent whenever an event involving a workspace occurs such as
    /// initialization of a new workspace or a different workspace gains
    /// focus
    Workspace = 0,
    /// Sent when outputs are updated
    Output = 1,
    /// Sent whenever the binding mode changes
    Mode = 2,
    /// Sent whenever an event involving a window occurs such as being
    /// reparented, focused, or closed
    Window = 3,
    /// Sent whenever a bar config changes
    #[serde(rename = "barconfig_update")]
    BarConfigUpdate = 4,
    /// Sent when a configured binding is executed
    Binding = 5,
    /// Sent when the ipc shuts down because sway is exiting
    Shutdown = 6,
    /// Sent when an ipc client sends a SEND_TICK message
    Tick = 7,
    /// Send when the visibility of a bar should change due to a modifier
    BarStateUpdate = 20,
    /// Sent when something related to input devices changes
    Input = 21,
}

impl TryFrom<u32> for EventType {
    type Error = CommandTypeError;

    fn try_from(n: u32) -> Result<EventType, Self::Error> {
        match n {
            0 => Ok(EventType::Workspace),
            1 => Ok(EventType::Output),
            2 => Ok(EventType::Mode),
            3 => Ok(EventType::Window),
            4 => Ok(EventType::BarConfigUpdate),
            5 => Ok(EventType::Binding),
            6 => Ok(EventType::Shutdown),
            7 => Ok(EventType::Tick),
            20 => Ok(EventType::BarStateUpdate),
            21 => Ok(EventType::Input),
            v => Err(CommandTypeError::IncorrectEventType(v)),
        }
    }
}
