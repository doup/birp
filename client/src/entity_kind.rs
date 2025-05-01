use crate::{EntityItem, component};

/// Components to guess the entity "kind"
pub const KIND_COMPONENTS: [&'static str; 11] = [
    component::CAMERA,
    component::LIGHT_DIRECTIONAL,
    component::LIGHT_POINT,
    component::LIGHT_SPOT,
    component::MESH_2D,
    component::MESH_3D,
    component::MONITOR,
    component::NODE,
    component::POINTER_ID,
    component::TEXT,
    component::WINDOW,
    // TODO(bevy_remote): See: https://github.com/bevyengine/bevy/issues/18869
    // component::OBSERVER_STATE,
    // component::SYSTEM_ID_MARKER,
];

#[derive(Debug)]
pub enum EntityKind {
    Camera,
    Entity,
    Light,
    Mesh2d,
    Mesh3d,
    Monitor,
    Node,
    Observer,
    Pointer,
    System,
    Text,
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
        } else if value.has_component(component::MONITOR) {
            EntityKind::Monitor
        } else if value.has_component(component::TEXT) {
            EntityKind::Text
        } else if value.has_component(component::NODE) {
            EntityKind::Node
        } else if value.has_component(component::OBSERVER_STATE) {
            EntityKind::Observer
        } else if value.has_component(component::POINTER_ID) {
            EntityKind::Pointer
        } else if value.has_component(component::SYSTEM_ID_MARKER) {
            EntityKind::System
        } else if value.has_component(component::WINDOW) {
            EntityKind::Window
        } else {
            EntityKind::Entity
        }
    }
}
