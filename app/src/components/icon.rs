use client::{EntityItem, EntityKind};
use dioxus::prelude::*;

pub enum Icon {
    Box,
    Bubbles,
    ChevronDown,
    ChevronRight,
    Cursor,
    Lightbulb,
    Refresh,
    Video,
    Window,
}

impl Icon {
    pub fn render(&self) -> Element {
        let (file_name, ratio) = match self {
            Icon::Box => ("box-3-line", 1.0),
            Icon::Bubbles => ("bubble-chart-line", 1.0),
            Icon::ChevronDown => ("arrow-down-s-line", 1.0),
            Icon::ChevronRight => ("arrow-right-s-line", 1.0),
            Icon::Cursor => ("cursor-line", 1.0),
            Icon::Lightbulb => ("lightbulb-line", 1.0),
            Icon::Refresh => ("refresh-right-fill", 1.0),
            Icon::Video => ("video-on-line", 1.0),
            Icon::Window => ("window-line", 1.0),
        };

        rsx! {
            i {
                class: "icon",
                style: "--icon-url: url(\"/assets/icons/{file_name}.svg\"); --icon-ratio: {ratio};",
            }
        }
    }
}

impl From<EntityKind> for Icon {
    fn from(kind: EntityKind) -> Self {
        match kind {
            EntityKind::Camera => Icon::Video,
            EntityKind::Light => Icon::Lightbulb,
            EntityKind::Mesh3d => Icon::Box,
            EntityKind::Window => Icon::Window,
            _ => Icon::Bubbles,
        }
    }
}

impl From<&EntityItem> for Icon {
    fn from(entity: &EntityItem) -> Self {
        EntityKind::from(entity).into()
    }
}
