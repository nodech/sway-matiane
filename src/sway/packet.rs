use bytes::Bytes;
use thiserror::Error;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SwayPacketType {
    Command(super::command::CommandType),
    Event(super::command::EventType),
}

#[derive(Debug)]
pub struct SwayPacketRaw {
    pub packet_type: u32,
    pub payload: Bytes,
}

#[derive(Error, Debug)]
pub enum SwayDeserializeError {
    #[error("invalid command type `{0}`")]
    InvalidCommandType(u32),
}

// #[derive(Debug)]
// pub struct SwayPacket {
//     pub command: super::command::SwayCommandType,
//     // pub payload::
