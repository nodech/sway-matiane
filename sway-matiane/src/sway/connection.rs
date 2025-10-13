use super::codec::{SwayPacketCodec, SwayPacketCodecError};
use super::command::{CommandType, CommandTypeError, EventType};
use super::packet::SwayPacketRaw;
use super::reply::{CommandOutcome, Event};
use futures::{SinkExt, StreamExt};
use log::debug;
use serde_json;
use std::fmt::Debug;
use std::path::PathBuf;
use thiserror::Error;
use tokio::net::UnixStream;
use tokio_util::codec::Framed;

#[derive(Debug, Error)]
pub enum SubscribeError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Incorrect response type.")]
    IncorrectResponseType,
    #[error("Subscribe command failed.")]
    SubscribeFailed(super::reply::CommandError),
    #[error("Subscribed response is not an event: `{0}`.")]
    NotAnEvent(u32),
    #[error("Unsupported event: `{0}`.")]
    UnsupportedEvent(u32),
    #[error("Terrible packet `{0}`.")]
    TerriblePacket(#[from] SwayPacketCodecError),
    #[error("Stream closed")]
    Closed,
    #[error("Bad payload")]
    BadPayload(#[from] serde_json::Error),
    #[error("Bad command type: {0}")]
    BadCommand(#[from] CommandTypeError),
}

impl TryFrom<SwayPacketRaw> for Event {
    type Error = SubscribeError;

    fn try_from(packet: SwayPacketRaw) -> Result<Event, SubscribeError> {
        if (packet.packet_type & super::EVENT_FLAG) != super::EVENT_FLAG {
            return Err(SubscribeError::NotAnEvent(packet.packet_type));
        }

        let event_type =
            EventType::try_from(packet.packet_type ^ super::EVENT_FLAG)?;

        match event_type {
            EventType::Window => {
                Ok(Event::Window(serde_json::from_slice(&packet.payload)?))
            }
            _ => Err(SubscribeError::UnsupportedEvent(event_type as u32)),
        }
    }
}

fn subscribe_packet(event: EventType) -> Result<SwayPacketRaw, SubscribeError> {
    let events = [event];
    let encoded = serde_json::ser::to_string(&events)?;

    Ok(SwayPacketRaw {
        packet_type: CommandType::Subscribe as u32,
        payload: encoded.into(),
    })
}

pub async fn subscribe(
    path: &PathBuf,
    event: EventType,
) -> Result<
    impl Debug + StreamExt<Item = Result<Event, SubscribeError>>,
    SubscribeError,
> {
    debug!("Connecting to {:?}", path);
    let socket = UnixStream::connect(path).await?;
    debug!("Connected to {:?}.", path);

    let mut framer = Framed::new(socket, SwayPacketCodec);

    debug!("Subscribing to events: {:?}", event);
    let packet = subscribe_packet(event)?;
    framer.send(packet).await?;

    let response = framer.next().await.ok_or(SubscribeError::Closed)??;

    if response.packet_type != (CommandType::Subscribe as u32) {
        return Err(SubscribeError::IncorrectResponseType);
    }

    let outcome: CommandOutcome = serde_json::de::from_slice(&response.payload)
        .map_err(SubscribeError::BadPayload)?;

    if !outcome.success {
        return Err(SubscribeError::SubscribeFailed(outcome.error.unwrap()));
    }

    debug!("Subscribed to events: {:?}.", event);
    Ok(framer.map(|res| Event::try_from(res?)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscribe_packet_test() -> anyhow::Result<()> {
        let packet = EventType::Window;

        subscribe_packet(packet)?;

        Ok(())
    }
}
