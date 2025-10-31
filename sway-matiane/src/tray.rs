use zbus::{
    interface,
    zvariant::{ObjectPath, Type, Value},
};

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
    async fn overlay_icon_pixmap(&self) -> Icon {
        Icon {
            width: 0,
            height: 0,
            data: vec![],
        }
    }

    #[zbus(property)]
    async fn attention_icon_name(&self) -> String {
        "".into()
    }

    #[zbus(property)]
    async fn attention_icon_pixmap(&self) -> Icon {
        Icon {
            width: 0,
            height: 0,
            data: vec![],
        }
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
