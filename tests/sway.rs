use anyhow::Result;
use futures::StreamExt;
use sway_matiane::sway::command::EventType;
use sway_matiane::sway::connection::subscribe;
use sway_matiane::sway::reply::{Event, WindowChange};
use tempfile::Builder;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;

#[tokio::test]
async fn sway_window_events_1() -> Result<()> {
    let dir = Builder::new()
        .prefix("sway-matiane-sway-")
        .rand_bytes(10)
        .tempdir()?;

    let server_recv = include_bytes!("data/send01.bin");
    let server_send = include_bytes!("data/received01-01.bin");

    let bind_path = dir.path().join("window-events-1.sock");
    let bind = UnixListener::bind(&bind_path)?;

    let server_task = tokio::spawn(async move {
        let (mut stream, _addr) = bind.accept().await?;

        let mut dup: Vec<u8> = vec![0; server_recv.len()];
        let read_res = stream.read_exact(&mut dup).await?;
        assert_eq!(read_res, server_recv.len());

        stream.write(server_send).await?;

        stream.shutdown().await?;

        Ok::<_, anyhow::Error>(stream)
    });

    let mut subbed = subscribe(&bind_path, EventType::Window).await?;
    let single_event = subbed.next().await.unwrap()?;

    if let Event::Window(window) = single_event {
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
        // ...
    } else {
        panic!("Must return a window.");
    }

    let second = subbed.next().await;
    matches!(second, None);

    server_task.await??;

    Ok(())
}
