use bevy_ecs::entity::Entity;
use bevy_remote::{
    builtin_methods::{BrpGetComponentsResponse, BrpQueryRow},
    error_codes::COMPONENT_ERROR,
};
use serde_json::Value;
use std::collections::BTreeMap;

use crate::{BrpError, component};

#[derive(Debug)]
pub struct EntityItem {
    pub id: Entity,
    pub components: BTreeMap<String, Option<Value>>,
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

impl From<(Entity, BrpGetComponentsResponse)> for EntityItem {
    fn from(from: (Entity, BrpGetComponentsResponse)) -> Self {
        let mut empty_components = vec![];
        let components = match from.1 {
            BrpGetComponentsResponse::Lenient { components, errors } => {
                // Some errors happen when the component is not
                // reflectable/serializable. We still want to list them.
                for (component, value) in errors.iter() {
                    let error = serde_json::from_value::<BrpError>(value.clone());

                    if let Ok(error) = error {
                        // This is OK as long as in `bevy/get` we don't query
                        // for NON-REFLECTABLE components that are not part of
                        // the entity
                        if error.code == COMPONENT_ERROR {
                            empty_components.push(component.clone());
                        }
                    }
                }

                components
            }
            BrpGetComponentsResponse::Strict(components) => components,
        };

        let mut components: BTreeMap<String, Option<Value>> = components
            .into_iter()
            .map(|(key, value)| (key, Some(value)))
            .collect();

        for component in empty_components {
            // Only insert `None` if the key is not already in `components`
            components.entry(component).or_insert(None);
        }

        Self {
            id: from.0,
            components,
        }
    }
}

impl From<BrpQueryRow> for EntityItem {
    fn from(from: BrpQueryRow) -> Self {
        let mut components: BTreeMap<String, Option<Value>> = from
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
