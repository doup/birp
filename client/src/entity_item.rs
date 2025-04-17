use bevy_ecs::entity::Entity;
use bevy_remote::builtin_methods::{BrpGetResponse, BrpQueryRow};
use serde_json::Value;
use std::collections::HashMap;

use crate::component;

#[derive(Debug)]
pub struct EntityItem {
    pub id: Entity,
    pub components: HashMap<String, Option<Value>>,
}

impl EntityItem {
    pub fn children(&self) -> Vec<Entity> {
        self.get_component_as::<Vec<Entity>>(component::CHILDREN)
            .unwrap_or_default()
    }

    pub fn get_component_as<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.components.get(key).and_then(|value| {
            value
                .clone()
                .and_then(|value| serde_json::from_value(value).ok())
        })
    }

    pub fn has_component(&self, component: &str) -> bool {
        self.components.contains_key(component)
    }

    pub fn name(&self) -> Option<String> {
        self.get_component_as::<String>(component::NAME)
    }
}

impl From<(Entity, BrpGetResponse)> for EntityItem {
    fn from(from: (Entity, BrpGetResponse)) -> Self {
        let components = match from.1 {
            // TODO: Check `errors``, as there are some errors that could be mapped to `None`?
            BrpGetResponse::Lenient { components, .. } => components,
            BrpGetResponse::Strict(components) => components,
        };

        let components: HashMap<String, Option<Value>> = components
            .into_iter()
            .map(|(key, value)| (key, Some(value)))
            .collect();

        Self {
            id: from.0,
            components,
        }
    }
}

impl From<BrpQueryRow> for EntityItem {
    fn from(from: BrpQueryRow) -> Self {
        let mut components: HashMap<String, Option<Value>> = from
            .components
            .into_iter()
            .map(|(key, value)| (key, Some(value)))
            .collect();

        for (key, value) in from.has {
            // Only insert `None` if the key is not already in `components` and the value is `true`
            if let Some(true) = value.as_bool() {
                components.entry(key).or_insert(None);
            }
        }

        Self {
            id: from.entity,
            components,
        }
    }
}
