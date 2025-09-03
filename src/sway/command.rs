use super::common::SwayPacketCodecError;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SwayCommandType {
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

impl TryFrom<i32> for SwayCommandType {
    type Error = SwayPacketCodecError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SwayCommandType::RunCommand),
            1 => Ok(SwayCommandType::GetWorkspaces),
            2 => Ok(SwayCommandType::Subscribe),
            3 => Ok(SwayCommandType::GetOutputs),
            4 => Ok(SwayCommandType::GetTree),
            5 => Ok(SwayCommandType::GetMarks),
            6 => Ok(SwayCommandType::GetBarConfig),
            7 => Ok(SwayCommandType::GetVersion),
            8 => Ok(SwayCommandType::GetBindingModes),
            9 => Ok(SwayCommandType::GetConfig),
            10 => Ok(SwayCommandType::SendTick),
            11 => Ok(SwayCommandType::Sync),
            12 => Ok(SwayCommandType::GetBindingState),
            100 => Ok(SwayCommandType::GetInputs),
            101 => Ok(SwayCommandType::GetSeats),
            _ => Err(SwayPacketCodecError::InvalidCommandType(value)),
        }
    }
}
