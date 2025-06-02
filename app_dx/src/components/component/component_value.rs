use client::{Entity, JsonSchemaBevyType, SchemaKind, SchemaType, Value, component, from_value};
use dioxus::prelude::*;

use crate::{
    bevy_type,
    components::{
        JsonValue,
        component::{
            value_bool::ValueBool, value_entity::ValueEntity, value_number::ValueNumber,
            value_select::ValueSelect, value_string::ValueString,
        },
    },
    states::ConnectionState,
    utils::{get_array_path, get_object_path, get_type_path_from_ref_value, value_to_string},
};

use super::{MutateData, map_value::map_value};

#[component]
pub fn ComponentValue(
    value: Value,
    component_type: String,
    bevy_type: JsonSchemaBevyType,
    mutate_cb: Callback<MutateData>,
    parent_path: Option<String>,
) -> Element {
    let schema = use_context::<ConnectionState>().schema;
    let value = map_value(&bevy_type.type_path, value);
    let path = use_signal(|| parent_path.clone().unwrap_or_default());
    let read_only = [
        component::COMPUTED_NODE_TARGET,
        component::COMPUTED_NODE,
        component::COMPUTED_TEXT_BLOCK,
        component::GLOBAL_TRANSFORM,
        component::POINTER_PRESS,
        component::MONITOR,
        component::TEXT_LAYOUT_INFO,
    ];

    if read_only.contains(&bevy_type.type_path.as_str()) {
        return rsx! {
            JsonValue { value: value.clone(), parent_path: path() }
        };
    }

    match bevy_type.type_path.as_str() {
        bevy_type::ENTITY => {
            let entity = from_value::<Entity>(value);

            match entity {
                Ok(entity) => rsx! {
                    ValueEntity { entity }
                },
                _ => rsx! {
                    div { class: "issue", "Invalid entity" }
                },
            }
        }
        component::NAME => rsx! {
            ValueString {
                value: value_to_string(&value),
                onchange: move |value| mutate_cb.call(MutateData::new(&component_type, path(), value)),
            }
        },
        // bevy_type::FLEX_DIRECTION => {
        //     rsx! { "Custom FlexDirection component" }
        // }
        _ => match (&bevy_type.schema_type, &bevy_type.kind) {
            (SchemaType::Boolean, SchemaKind::Value) => rsx! {
                ValueBool {
                    value: value.as_bool().unwrap_or_default(),
                    onchange: move |value| mutate_cb.call(MutateData::new(&component_type, path(), value)),
                }
            },
            (SchemaType::Float, SchemaKind::Value)
            | (SchemaType::Int, SchemaKind::Value)
            | (SchemaType::Uint, SchemaKind::Value) => rsx! {
                ValueNumber {
                    value: value_to_string(&value),
                    schema_type: bevy_type.schema_type.clone(),
                    onchange: move |value| mutate_cb.call(MutateData::new(&component_type, path(), value)),
                }
            },
            (SchemaType::String, SchemaKind::Value) => rsx! {
                ValueString {
                    value: value_to_string(&value),
                    onchange: move |value| mutate_cb.call(MutateData::new(&component_type, path(), value)),
                }
            },
            (SchemaType::String, SchemaKind::Enum) => rsx! {
                ValueSelect {
                    value: value_to_string(&value),
                    options: bevy_type.one_of.iter().map(value_to_string).collect(),
                    onchange: move |value| mutate_cb.call(MutateData::new(&component_type, path(), value)),
                }
            },
            (SchemaType::Array, SchemaKind::List) => {
                let value = value.as_array().cloned().unwrap_or_else(Vec::new);
                let type_ref = bevy_type.items.unwrap();
                let bevy_type = get_type_path_from_ref_value(&type_ref)
                    .and_then(|type_path| schema().get(&type_path).cloned());

                match bevy_type {
                    Some(bevy_type) => rsx! {
                        table { class: "json-value-table json-value-table--array",
                            if value.is_empty() {
                                div { class: "json-value-empty", "Empty Array" }
                            } else {
                                for (idx , item) in value.iter().enumerate() {
                                    {
                                        let item_path = get_array_path(&parent_path, idx.to_string().as_str());
                                        rsx! {
                                            tr {
                                                th { title: "{item_path}", "⚬" }
                                                td {
                                                    ComponentValue {
                                                        value: item.clone(),
                                                        component_type: component_type.clone(),
                                                        bevy_type: bevy_type.clone(),
                                                        mutate_cb,
                                                        parent_path: item_path.clone(),
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    None => rsx! {
                        div { class: "issue", "Type not found: {type_ref:?}" }
                    },
                }
            }
            (SchemaType::Array, SchemaKind::TupleStruct) => {
                let type_ref = bevy_type.prefix_items.first().unwrap();
                let parent_path = get_object_path(&parent_path, "0");
                let bevy_type = get_type_path_from_ref_value(type_ref)
                    .and_then(|type_path| schema().get(&type_path).cloned());

                match bevy_type {
                    Some(bevy_type) => rsx! {
                        ComponentValue {
                            value,
                            component_type,
                            bevy_type,
                            mutate_cb,
                            parent_path,
                        }
                    },
                    None => rsx! {
                        div { class: "issue", "Type not found: {type_ref:?}" }
                    },
                }
            }
            //     }
            // },
            (SchemaType::Object, SchemaKind::Struct) => {
                let mut properties: Vec<_> = bevy_type
                    .properties
                    .iter()
                    .map(|(k, v)| {
                        (
                            k,
                            get_type_path_from_ref_value(v)
                                .and_then(|type_path| schema().get(&type_path).cloned()),
                        )
                    })
                    .collect();

                properties.sort_by_key(|(key, _)| *key);

                // TODO: Add same `has_subobjects` optimization as in `JsonValue` component

                rsx! {
                    div { class: "json-value-key-list",
                        for (key , prop_type) in properties {
                            {
                                match prop_type {
                                    Some(prop_type) => {
                                        let value = value.as_object().and_then(|obj| obj.get(key));
                                        rsx! {
                                            div { key, class: "json-value-key-list__item",
                                                div { class: "json-value-key-list__key", "{key}" }
                                                div { class: "json-value-key-list__value",
                                                    {
                                                        match value {
                                                            Some(value) => rsx! {
                                                                ComponentValue {
                                                                    value: value.clone(),
                                                                    component_type: component_type.clone(),
                                                                    bevy_type: prop_type.clone(),
                                                                    mutate_cb,
                                                                    parent_path: get_object_path(&parent_path, key),
                                                                }
                                                            },
                                                            None => rsx! {
                                                                div { class: "issue", "Can't unwrap value." }
                                                            },
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    None => rsx! {
                                        div { class: "issue", "Unknown property type." }
                                    },
                                }
                            }
                        }
                    }
                }
            }
            _ => rsx! {
                JsonValue { value }
            },
        },
    }
}
