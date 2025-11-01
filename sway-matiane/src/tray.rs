use zbus::{
    Connection, connection,
    fdo::DBusProxy,
    interface,
    zvariant::{ObjectPath, Type, Value},
};

use futures::StreamExt;
use std::time::Duration;
use tokio::task::{JoinHandle, spawn};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use log::debug;
use thiserror::Error;

pub struct Tray;

const ICON_WIDTH: i32 = 256;
const ICON_HEIGHT: i32 = 256;
const ICON_DATA: &[u8] = include_bytes!("../res/logo-256x256.icon");

#[derive(Value, Type)]
struct Icon {
    width: i32,
    height: i32,
    data: Vec<u8>,
}

#[derive(Value, Type)]
struct ToolTip {
    icon_name: String,
    icon_pixmap: Vec<Icon>,
    title: String,
    description: String,
}

#[interface(name = "org.kde.StatusNotifierItem")]
impl Tray {
    #[zbus(property)]
    async fn category(&self) -> String {
        "SystemServices".into()
    }

    #[zbus(property)]
    async fn id(&self) -> String {
        matiane_core::NAME.into()
    }

    #[zbus(property)]
    async fn title(&self) -> String {
        "".into()
    }

    #[zbus(property)]
    async fn status(&self) -> String {
        "Active".into()
    }

    #[zbus(property)]
    async fn window_id(&self) -> i32 {
        0
    }

    #[zbus(property)]
    async fn icon_theme_path(&self) -> String {
        "".into()
    }

    #[zbus(property)]
    async fn item_is_menu(&self) -> bool {
        false
    }

    #[zbus(property)]
    async fn menu(&self) -> ObjectPath<'_> {
        ObjectPath::from_static_str_unchecked("/")
    }

    #[zbus(property)]
    async fn icon_name(&self) -> String {
        "something-something".into()
    }

    #[zbus(property)]
    async fn icon_pixmap(&self) -> Vec<Icon> {
        let icon = Icon {
            width: ICON_WIDTH,
            height: ICON_HEIGHT,
            data: ICON_DATA.to_vec(),
        };

        vec![icon]
    }

    #[zbus(property)]
    async fn overlay_icon_name(&self) -> String {
        "".into()
    }

    #[zbus(property)]
    async fn overlay_icon_pixmap(&self) -> Vec<Icon> {
        vec![Icon {
            width: 0,
            height: 0,
            data: vec![],
        }]
    }

    #[zbus(property)]
    async fn attention_icon_name(&self) -> String {
        "".into()
    }

    #[zbus(property)]
    async fn attention_icon_pixmap(&self) -> Vec<Icon> {
        vec![Icon {
            width: 0,
            height: 0,
            data: vec![],
        }]
    }

    #[zbus(property)]
    async fn tool_tip(&self) -> ToolTip {
        ToolTip {
            icon_name: "".into(),
            icon_pixmap: vec![],
            title: matiane_core::NAME.into(),
            description: "Activity logger.".into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum TrayError {
    #[error("DBus error: {0}")]
    DBusError(#[from] zbus::Error),
}

pub enum TrayState {
    Offline,
    Uninitialized,
    Initialized,
}

struct TrayConn<'a> {
    conn: &'a Connection,
    state: TrayState,
}

impl TrayConn<'_> {
    async fn register(&mut self) -> Result<zbus::Message, TrayError> {
        let path = self.conn.unique_name().unwrap();
        let message = self
            .conn
            .call_method(
                Some("org.kde.StatusNotifierWatcher"),
                "/StatusNotifierWatcher",
                Some("org.kde.StatusNotifierWatcher"),
                "RegisterStatusNotifierItem",
                &path,
            )
            .await?;

        self.state = TrayState::Initialized;

        debug!("Registering item at path: {}", path);

        Ok(message)
    }
}

pub fn spawn_tray(
    token: CancellationToken,
) -> JoinHandle<Result<(), anyhow::Error>> {
    spawn(async move {
        let connection = connection::Builder::session()?
            .serve_at("/StatusNotifierItem", Tray)?
            .build()
            .await?;

        let mut tcon = TrayConn {
            conn: &connection,
            state: TrayState::Uninitialized,
        };

        let dbus = DBusProxy::new(&connection).await?;
        let mut change_signal = dbus
            .receive_name_owner_changed_with_args(&[(
                0,
                "org.kde.StatusNotifierWatcher",
            )])
            .await?;

        loop {
            match tcon.state {
                TrayState::Offline | TrayState::Initialized => {}
                TrayState::Uninitialized => {
                    if tcon.register().await.is_err() {
                        // consider incrementing time every time
                        // it fails, with upper limit.
                        sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                }
            }

            // watch dbus Owner change event
            tokio::select! {
                _ = token.cancelled() => {
                    debug!("Shutting down.");
                    break;
                },
                chsignal = change_signal.next() => {
                    if chsignal.is_none() {
                        continue;
                    }

                    let changed = chsignal.unwrap();
                    let args = changed.args().unwrap();

                    if args.new_owner.is_none() {
                        tcon.state = TrayState::Offline;
                    } else {
                        tcon.state = TrayState::Uninitialized;
                    }
                },
            }
        }

        Ok(())
    })
}
