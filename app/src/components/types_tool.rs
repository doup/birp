use client::Value;
use dioxus::prelude::*;

use crate::{
    components::JsonValue,
    states::{ConnectionState, TypesToolState},
};

#[component]
pub fn TypesTool() -> Element {
    let schema = use_context::<ConnectionState>().schema;
    let mut active = use_context::<TypesToolState>().active;
    let mut filter = use_context::<TypesToolState>().filter;
    let row_click = |ty: String| {
        move |_| {
            active.set(Some(ty.clone()));
        }
    };

    rsx! {
        div { class: "types-tool",
            div { class: "types-tool__sidebar card",
                div { class: "types-tool__sidebar-header",
                    input {
                        class: "types-tool__filter",
                        name: "type-filter",
                        value: filter(),
                        autocomplete: "off",
                        oninput: move |e| filter.set(e.data.value().to_lowercase()),
                    }
                }

                div { class: "types-tool__types",
                    for (ty , schema) in schema().iter() {
                        if filter().is_empty() || ty.to_lowercase().contains(&filter()) {
                            div {
                                class: "types-tool__type",
                                onclick: row_click(ty.clone()),
                                // Replace '<' with '<' followed by a Zero-Width Space
                                {schema.short_path.replace('<', "<\u{200B}")}
                            }
                        }
                    }
                }
            }

            div { class: "types-tool__schema",
                if let Some(active_ty) = active() {
                    if let Some(schema) = schema().get(&active_ty) {
                        div { class: "type",
                            div {
                                class: "type__name",
                                title: "{schema.type_path}",
                                "{schema.short_path}"
                            }
                            div { class: "type__schema",
                                table { class: "json-value-table",
                                    // if let Some(crate_name) = &schema.crate_name {
                                    //     tr {
                                    //         th { "Crate Name" }
                                    //         td { "{crate_name}" }
                                    //     }
                                    // }
                                    if let Some(module) = &schema.module_path {
                                        tr {
                                            th { "Module" }
                                            td {
                                                JsonValue { value: Value::from(module.clone()) }
                                            }
                                        }
                                    }
                                    if !schema.reflect_types.is_empty() {
                                        tr {
                                            th { "Reflect Types" }
                                            td {
                                                JsonValue { value: Value::from(schema.reflect_types.clone()) }
                                            }
                                        }
                                    }
                                    tr {
                                        th { "Kind" }
                                        td {
                                            JsonValue { value: Value::from(format!("{:?}", schema.kind)) }
                                        }
                                    }
                                    tr {
                                        th { "Schema Type" }
                                        td {
                                            JsonValue { value: Value::from(format!("{:?}", schema.schema_type)) }
                                        }
                                    }
                                    if !schema.one_of.is_empty() {
                                        tr {
                                            th { "One Of" }
                                            td {
                                                JsonValue { value: Value::from(schema.one_of.clone()) }
                                            }
                                        }
                                    }
                                    if let Some(additional_properties) = &schema.additional_properties {
                                        tr {
                                            th { "Additional Properties" }
                                            td {
                                                JsonValue { value: Value::from(*additional_properties) }
                                            }
                                        }
                                    }
                                    if !schema.properties.is_empty() {
                                        tr {
                                            th { "Properties" }
                                            td {
                                                table { class: "json-value-table",
                                                    for (key , value) in schema.properties.iter() {
                                                        tr {
                                                            th {
                                                                "{key}"
                                                                if schema.required.contains(key) {
                                                                    span {
                                                                        style: "color: red; font-weight: normal !important;",
                                                                        title: "Required",
                                                                        "*"
                                                                    }
                                                                }
                                                            }
                                                            td {
                                                                JsonValue { value: value.clone() }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    if let Some(key_type) = &schema.key_type {
                                        tr {
                                            th { "Key Type" }
                                            td {
                                                JsonValue { value: key_type.clone() }
                                            }
                                        }
                                    }
                                    if let Some(value_type) = &schema.value_type {
                                        tr {
                                            th { "Value Type" }
                                            td {
                                                JsonValue { value: value_type.clone() }
                                            }
                                        }
                                    }
                                    if !schema.prefix_items.is_empty() {
                                        tr {
                                            th { "Prefix Items" }
                                            td {
                                                JsonValue { value: Value::from(schema.prefix_items.clone()) }
                                            }
                                        }
                                    }
                                    if let Some(items) = &schema.items {
                                        tr {
                                            th { "Items" }
                                            td {
                                                JsonValue { value: items.clone() }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
