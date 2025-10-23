use client::{Entity, Value, component};
use dioxus::{logger::tracing::info, prelude::*};

use crate::{
    components::{ComponentValue, Icon},
    states::ConnectionState,
    utils::get_short_type_name,
};

#[derive(Debug)]
pub struct MutateData {
    /// Component type path, e.g. `bevy_ui::ui_node::Node`
    type_path: String,
    /// Path to the field in the component, e.g. `.align_content`
    path: String,
    value: Value,
}

impl MutateData {
    pub fn new(
        type_path: impl Into<String>,
        path: impl Into<String>,
        value: impl Into<Value>,
    ) -> Self {
        Self {
            type_path: type_path.into(),
            path: path.into(),
            value: value.into(),
        }
    }
}

#[component]
pub fn ComponentInspector(id: Entity, type_path: String, value: Option<Value>) -> Element {
    let client = use_context::<ConnectionState>().client;
    let schema = use_context::<ConnectionState>().schema;
    let mut is_open = use_signal(|| {
        ![
            component::COMPUTED_NODE,
            component::COMPUTED_TEXT_BLOCK,
            component::COMPUTED_UI_TARGET_CAMERA,
            component::GLOBAL_TRANSFORM,
            component::LIGHT_CASCADES,
            component::TEXT_LAYOUT_INFO,
        ]
        .contains(&type_path.as_str())
    });
    let header_class = use_memo(move || {
        format!(
            "component__header {}",
            if is_open() {
                "component__header--open"
            } else {
                "component__header--closed"
            }
        )
    });
    let bevy_type = use_memo({
        let type_path = type_path.clone();
        move || schema().get(&type_path).cloned()
    });
    let mutate_cb = use_callback(move |data: MutateData| {
        info!("MutateData: {data:?}");
        spawn(async move {
            let _ = client()
                .mutate_component(id, data.type_path, data.path, data.value)
                .await;
        });
    });

    rsx! {
        div { class: "component",
            div {
                class: header_class(),
                title: "{type_path}",
                onclick: move |_| is_open.set(!is_open()),
                span { {get_short_type_name(&type_path)} }
                {Icon::ChevronDown.render_with_class("component__open-icon")}
            }

            div { class: "component__value",
                if is_open() {
                    {
                        match bevy_type() {
                            Some(bevy_type) => {
                                if let Some(value) = value {
                                    rsx! {
                                        ComponentValue {
                                            value: value.clone(),
                                            component_type: bevy_type.type_path.clone(),
                                            bevy_type: bevy_type.clone(),
                                            mutate_cb,
                                        }
                                    }
                                } else {
                                    rsx! {
                                        div { class: "issue issue--no-value", "No value." }
                                    }
                                }
                            }
                            None => rsx! {
                                div { class: "issue", "Type not registered." }
                            },
                        }
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
