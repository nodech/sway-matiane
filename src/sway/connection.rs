use super::codec::{SwayPacketCodec, SwayPacketCodecError};
use super::command::{CommandType, EventType};
use super::packet::SwayPacketRaw;
use super::reply::{CommandOutcome, Event};
use anyhow::Result;
use futures::{SinkExt, StreamExt};
use serde_json;
use thiserror::Error;
use tokio::net::UnixStream;
use tokio_util::codec::Framed;

#[derive(Debug, Error)]
pub enum SubscribeError {
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
}

impl TryFrom<SwayPacketRaw> for Event {
    type Error = anyhow::Error;

    fn try_from(packet: SwayPacketRaw) -> Result<Event> {
        if (packet.packet_type & super::EVENT_FLAG) == super::EVENT_FLAG {
            return Err(SubscribeError::NotAnEvent(packet.packet_type).into());
        }

        let event_type = EventType::try_from(packet.packet_type ^ super::EVENT_FLAG)?;

        match event_type {
            EventType::Window => Ok(Event::Window(serde_json::from_slice(&packet.payload)?)),
            _ => Err(SubscribeError::UnsupportedEvent(event_type as u32).into()),
        }
    }
}

fn subscribe_packet(event: EventType) -> Result<SwayPacketRaw> {
    let events = [event];
    let encoded = serde_json::ser::to_string(&events)?;

    Ok(SwayPacketRaw {
        packet_type: CommandType::Subscribe as u32,
        payload: encoded.into(),
    })
}

pub async fn subscribe(
    path: &str,
    event: EventType,
) -> Result<impl StreamExt<Item = Result<Event, anyhow::Error>>> {
    let socket = UnixStream::connect(path).await?;
    let mut framer = Framed::new(socket, SwayPacketCodec);

    let packet = subscribe_packet(event)?;
    framer.send(packet).await?;

    let response = framer.next().await.expect("Must receive response.")?;

    if response.packet_type != (CommandType::Subscribe as u32) {
        return Err(SubscribeError::IncorrectResponseType.into());
    }

    let outcome: CommandOutcome = serde_json::de::from_slice(&response.payload)?;

    if !outcome.success {
        return Err(SubscribeError::SubscribeFailed(outcome.error.unwrap()).into());
    }

    Ok(framer.map(move |res| {
        let raw = res?;

        if raw.packet_type != event as u32 {
            return Err(SubscribeError::IncorrectResponseType.into());
        }

        Event::try_from(raw)
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscribe_packet_test() -> Result<()> {
        let packet = EventType::Window;

        subscribe_packet(packet)?;

        Ok(())
    }
}
