use client::{SchemaType, Value};
use dioxus::prelude::*;

use crate::{components::JsonValue, states::ConnectionState, utils::get_short_type_name};

#[component]
pub fn ComponentInspector(type_path: String, value: Option<Value>) -> Element {
    let schema = use_context::<ConnectionState>().schema;
    let bevy_type = use_memo({
        let type_path = type_path.clone();
        move || schema().get(&type_path.clone()).cloned()
    });

    rsx! {
        div { class: "component",
            div { class: "component__header", title: "{type_path}",
                {get_short_type_name(&type_path)}
            }

            div { class: "component__value",
                {
                    match bevy_type() {
                        Some(bevy_type) => {
                            if let Some(value) = value {
                                rsx! {
                                    // div { style: "font-size: 10px;", "{bevy_type.schema_type:?} {bevy_type.kind:?}" }
                                    JsonValue { value }
                                }
                            } else {
                                rsx! {
                                    div { class: "component__no-value", "No value." }
                                }
                            }
                        }
                        None => rsx! {
                            div { class: "component__not-registered", "Type not registered." }
                        },
                    }
                }
            }
        }
    }
}

// match bevy_type.kind {
//     SchemaKind::TupleStruct => rsx! {
//         "TupleStruct"
//         if let Some(value) = value {
//             JsonValue { value }
//         }
//     },
//     SchemaKind::Struct => rsx! {
//         "Struct"
//         if let Some(value) = value {
//             JsonValue { value }
//         }
//     },
//     SchemaKind::Enum => {
//         match bevy_type.schema_type {
//             SchemaType::String => rsx! {
//                 select {
//                     for variant in bevy_type.one_of.iter() {
//                         option { value: "{variant}", "{variant}" }
//                     }
//                 }
//             },
//             _ => rsx! {
//             "TODO: {bevy_type.schema_type:?} Enum"
//             },
//         }
//     }
//     _ => rsx! {
//     "Unsupported: {bevy_type.kind:?}"
//     },
// }
