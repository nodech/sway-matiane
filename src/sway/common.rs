use thiserror::Error;
use bytes::Bytes;

/// Sway IPC magic string - "i3-ipc"
pub const MAGIC: [u8; 6] = *b"i3-ipc";
pub const MAGIC_LEN: usize = 6;

#[derive(Debug)]
pub struct SwayPacketRaw {
    pub packet_type: super::command::SwayCommandType,
    pub payload: Bytes,
}

#[derive(Error, Debug)]
pub enum SwayPacketCodecError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("magic inncorret")]
    MagicIncorrect,
    #[error("invalid payload len.")]
    PayloadLenIncorrect,
    #[error("invalid command type `{0}`")]
    InvalidCommandType(i32),
}
