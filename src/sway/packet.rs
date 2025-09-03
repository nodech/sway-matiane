use tokio_util::codec::{Decoder, Encoder};
use bytes::{BytesMut, Buf, BufMut};
use std::io::Cursor;

use super::common::{self, SwayPacketCodecError, SwayPacketRaw};

pub struct SwayPacketCodec;

impl Decoder for SwayPacketCodec {
    type Item = SwayPacketRaw;
    type Error = SwayPacketCodecError;

    fn decode(
        &mut self,
        src: &mut BytesMut
    ) -> Result<Option<Self::Item>, Self::Error> {
        let header_len = common::MAGIC_LEN + 4 + 4;

        if src.len() < header_len {
            return Ok(None);
        }

        if src[0..common::MAGIC_LEN] != common::MAGIC {
            src.clear();
            return Err(SwayPacketCodecError::MagicIncorrect);
        }

        let mut cursor = Cursor::new(&src[common::MAGIC_LEN..]);
        let payload_len = cursor.get_i32_ne();

        if payload_len < 0 {
            src.clear();
            return Err(SwayPacketCodecError::PayloadLenIncorrect)
        }

        let payload_len = payload_len as usize;
        let payload_type = cursor.get_i32_ne();

        if cursor.remaining() < payload_len {
            return Ok(None)
        }

        let mut packet = src.split_to(header_len + payload_len);
        packet.advance(header_len);

        Ok(Some(SwayPacketRaw{
            packet_type: payload_type.try_into()?,
            payload: packet.into(),
        }))
    }
}

impl Encoder<SwayPacketRaw> for SwayPacketCodec {
    type Error = SwayPacketCodecError;

    fn encode(
        &mut self,
        item: SwayPacketRaw,
        dst: &mut BytesMut
    ) -> Result<(), SwayPacketCodecError> {
        let header_len = common::MAGIC_LEN + 4 + 4;
        let payload_len = item.payload.len();

        dst.reserve(header_len + payload_len);
        dst.extend_from_slice(&common::MAGIC);
        dst.put_i32_ne(payload_len as i32);
        dst.put_i32_ne(item.packet_type as i32);
        dst.extend_from_slice(&item.payload);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use futures::{StreamExt, SinkExt};
    use tokio_test::io::Builder;
    use tokio_util::codec::Framed;
    use super::super::command::SwayCommandType;

    #[tokio::test]
    async fn decode_incomplete() {
        let mock = Builder::new()
            .read(b"i3-ip")
            .build();

        let mut framed = Framed::new(mock, SwayPacketCodec);

        let message = framed.next().await.expect("Should receive an error.");

        match message {
            Err(SwayPacketCodecError::Io(err)) => {
                assert_eq!(err.kind(), std::io::ErrorKind::Other);
                let str = err.to_string();
                assert!(str.contains("bytes remaining on stream"),
                    "Expected bytes remaining error, got {:?}.", str);
            }
            _ => {
                panic!("Expected Io error, got: {:?}", &message);
            }
        }
    }

    #[tokio::test]
    async fn decode_incorrect_magic() {
        let mock = Builder::new()
            .read(b"i3-ipx")
            .read(b"12341234")
            .build();

        let mut framed = Framed::new(mock, SwayPacketCodec);

        let message = framed.next().await.expect("Should receive an error.");

        assert!(matches!(message, Err(SwayPacketCodecError::MagicIncorrect)),
            "Expected magic incorrect error, received: {:?}", message);
    }

    #[tokio::test]
    async fn decode_normal() {
        let payload: &[u8] = b"{}";
        let payload_type = SwayCommandType::GetTree;
        let payload2: &[u8] = b"something_else";
        let payload2_type = SwayCommandType::GetWorkspaces;

        let mock = Builder::new()
            .read(b"i3-ipc")
            .read(&(payload.len() as i32).to_ne_bytes())
            .read(&(payload_type as i32).to_ne_bytes())
            .read(payload)
            .read(b"i3-ipc")
            .read(&(payload2.len() as i32).to_ne_bytes())
            .read(&(payload2_type as i32).to_ne_bytes())
            .read(payload2)
            .build();

        let mut framed = Framed::new(mock, SwayPacketCodec);

        let message = framed.next().await.expect("Should receive a packet.");
        let packet = message.expect("We must get a packet.");
        assert_eq!(packet.packet_type, payload_type);
        assert_eq!(packet.payload, payload);

        let message = framed.next().await.expect("Should receive a second packet.");
        let packet = message.expect("We must get a packet.");
        assert_eq!(packet.packet_type, payload2_type);
        assert_eq!(packet.payload, payload2);
    }

    /// Tests decode to be cancel safe.
    #[tokio::test]
    async fn decode_cancel_continue() {
        let payload: &[u8] = b"{}";
        let payload_type = SwayCommandType::GetWorkspaces;

        let mock = Builder::new()
            .read(b"i3-ipc")
            .read(&(payload.len() as i32).to_ne_bytes())
            .read(&(payload_type as i32).to_ne_bytes())
            .wait(Duration::from_millis(50))
            .read(payload)
            .build();

        let mut framed = Framed::new(mock, SwayPacketCodec);

        #[derive(Debug, PartialEq)]
        enum Winner {
            Frame,
            Sleep,
        }

        let result = tokio::select! {
            _ = framed.next() => Winner::Frame,
            _ = tokio::time::sleep(Duration::from_millis(20)) => Winner::Sleep,
        };

        assert_eq!(result, Winner::Sleep);

        let again = framed.next().await.expect("Must receive a packet.");
        let packet = again.expect("We must get a packet.");
        assert_eq!(packet.packet_type, payload_type);
        assert_eq!(packet.payload, payload);
    }

    #[tokio::test]
    async fn encode() {
        let payload: &[u8] = b"{}";
        let payload_type = SwayCommandType::SendTick;
        let payload2: &[u8] = b"something_else";
        let payload2_type = SwayCommandType::GetSeats;

        let mock = Builder::new()
            .write(b"i3-ipc")
            .write(&(payload.len() as i32).to_ne_bytes())
            .write(&(payload_type as i32).to_ne_bytes())
            .write(payload)
            .write(b"i3-ipc")
            .write(&(payload2.len() as i32).to_ne_bytes())
            .write(&(payload2_type as i32).to_ne_bytes())
            .write(payload2)
            .build();

        let mut framed = Framed::new(mock, SwayPacketCodec);

        let send = framed.send(SwayPacketRaw {
            packet_type: payload_type,
            payload: payload.into()
        }).await;

        send.expect("Must send the packet.");

        let send = framed.send(SwayPacketRaw {
            packet_type: payload2_type,
            payload: payload2.into()
        }).await;

        send.expect("Must send the packet.");

    }
}
