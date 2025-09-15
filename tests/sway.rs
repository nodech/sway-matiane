use anyhow::Result;
use futures::StreamExt;
use serde_json;
use std::path::PathBuf;
use sway_matiane::sway::codec::SwayPacketCodecError;
use sway_matiane::sway::command::EventType;
use sway_matiane::sway::connection::{SubscribeError, subscribe};
use sway_matiane::sway::reply::{Event, WindowChange};
use tempfile::{Builder, TempDir};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::task::JoinHandle;

mod util;

#[cfg(target_endian = "little")]
#[tokio::test]
async fn sway_window_events_1() -> Result<()> {
    let server_recv = include_bytes!("data/send01.bin").to_vec();
    let server_send = include_bytes!("data/receive1.bin").to_vec();

    let MockServer {
        dir: _dir,
        bind_path,
        handle,
    } = setup_mock_server("window-events-1", server_recv, server_send)?;

    let mut subbed = subscribe(&bind_path, EventType::Window).await?;
    let single_event = subbed.next().await.unwrap()?;

    let Event::Window(window) = single_event else {
        panic!("Returned event must be a Window.");
    };

    assert_eq!(window.change, WindowChange::FullscreenMode);
    assert_eq!(window.container.id, 10);
    assert_eq!(
        window.container.name,
        Some(String::from(
            "Alacritty - dev-1 // 2 - zsh // 1 - sway-matiane/src"
        ))
    );
    assert_eq!(window.container.app_id, Some(String::from("Alacritty")));
    assert_eq!(window.container.rect.x, 0);
    assert_eq!(window.container.rect.y, 2185);
    assert_eq!(window.container.rect.width, 2880);
    assert_eq!(window.container.rect.height, 1775);

    let second = subbed.next().await;
    matches!(second, None);

    handle.await??;

    Ok(())
}

generate_sway_bad_subscribe_tests![
    [
        sway_subscribe_bad_magic,
        raw_packet![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, (u32_ne 0), (u32_ne 1)],
        SwayPacketCodecError,
        SwayPacketCodecError::MagicIncorrect
    ],
    [
        sway_subscribe_bad_payload_len,
        raw_packet![magic, [be2ne_4 0x80, 0x00, 0x00, 0x01], (u32_ne 0)],
        SwayPacketCodecError,
        SwayPacketCodecError::PayloadLenIncorrect
    ],
    [
        sway_subscribe_bad_type,
        raw_packet_with_body! {
            header: [magic, (u32_ne 2), (u32_ne 0)],
            body: br#"{}"#
        },
        SubscribeError,
        SubscribeError::IncorrectResponseType
    ],
    [
        sway_subscribe_bad_payload,
        raw_packet_with_body! {
            header: [magic, (u32_ne 2), (u32_ne 2)],
            body: br#"{}"#
        },
        SubscribeError,
        SubscribeError::BadPayload(_)
    ]
];

generate_sway_bad_event_tests![
    [
        sway_bad_event_magic,
        raw_packet![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, (u32_ne 0), (u32_ne 1)],
        SwayPacketCodecError::MagicIncorrect
    ],
    [
        sway_bad_event_payload_len,
        raw_packet![magic, [be2ne_4 0x80, 0x00, 0x00, 0x01], (u32_ne 0)],
        SwayPacketCodecError::PayloadLenIncorrect
    ],
];

struct MockServer {
    dir: TempDir,
    bind_path: PathBuf,
    handle: JoinHandle<Result<UnixStream>>,
}

fn setup_mock_server(
    name: &str,
    expect_recv: Vec<u8>,
    send: Vec<u8>,
) -> Result<MockServer> {
    let dir = Builder::new()
        .prefix(&format!("sway-matiane-{}", name))
        .rand_bytes(10)
        .tempdir()?;

    let bind_path = dir.path().join("window-events-1.sock");
    let bind = UnixListener::bind(&bind_path)?;

    let handle = tokio::spawn(async move {
        let (mut stream, _addr) = bind.accept().await?;

        let mut dup: Vec<u8> = vec![0; expect_recv.len()];
        let read_res = stream.read_exact(&mut dup).await?;
        assert_eq!(read_res, expect_recv.len());
        assert_eq!(dup, expect_recv);

        stream.write(&send).await?;
        stream.shutdown().await?;

        Ok::<_, anyhow::Error>(stream)
    });

    Ok(MockServer {
        dir,
        bind_path,
        handle,
    })
}
