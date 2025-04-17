use crate::{EntityItem, component};

#[derive(Debug)]
pub enum EntityKind {
    Camera,
    Light,
    Mesh2d,
    Mesh3d,
    Observer,
    Unknown,
    Window,
}

impl From<&EntityItem> for EntityKind {
    fn from(value: &EntityItem) -> Self {
        if value.has_component(component::CAMERA) {
            EntityKind::Camera
        } else if value.has_component(component::LIGHT_DIRECTIONAL)
            || value.has_component(component::LIGHT_POINT)
            || value.has_component(component::LIGHT_SPOT)
        {
            EntityKind::Light
        } else if value.has_component(component::MESH_2D) {
            EntityKind::Mesh2d
        } else if value.has_component(component::MESH_3D) {
            EntityKind::Mesh3d
        } else if value.has_component(component::WINDOW) {
            EntityKind::Window
        } else {
            EntityKind::Unknown
        }
    }
}
