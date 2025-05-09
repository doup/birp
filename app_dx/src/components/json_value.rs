use client::Value;
use dioxus::prelude::*;

#[component]
pub fn JsonValue(value: Value, parent_path: Option<String>) -> Element {
    match value {
        Value::Null => rsx! {
            span { class: "json-value json-value--null", "Null" }
        },
        Value::Bool(b) => {
            let modifier = if b { "true" } else { "false" };

            rsx! {
                span { class: "json-value json-value--{modifier}",
                    if b {
                        "True"
                    } else {
                        "False"
                    }
                }
            }
        }
        Value::Number(n) => {
            let mut class = "json-value json-value--number-positive";

            if let Some(n) = n.as_f64() {
                if n.is_sign_negative() {
                    class = "json-value json-value--number-negative";
                }
            }

            rsx! {
                span { class, "{n}" }
            }
        }
        Value::String(s) => rsx! {
            span { class: "json-value json-value--string", "\"{s}\"" }
        },
        Value::Array(arr) => rsx! {
            table { class: "json-value-table json-value-table--array",
                if arr.is_empty() {
                    div { class: "json-value-empty", "Empty Array" }
                } else {
                    for (idx , item) in arr.iter().enumerate() {
                        {
                            let item_path = get_array_path(&parent_path, idx.to_string().as_str());
                            rsx! {
                                tr {
                                    th { title: "{item_path}", "⚬" }
                                    td {
                                        JsonValue { value: item.clone(), parent_path: Some(item_path.clone()) }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        Value::Object(obj) => {
            let has_subobjects = obj
                .iter()
                .any(|(_, v)| matches!(v, Value::Object(_) | Value::Array(_)));

            if has_subobjects {
                rsx! {
                    div { class: "json-value-key-list",
                        if obj.is_empty() {
                            div { class: "json-value-empty", "Empty Object" }
                        } else {
                            for (key , value) in obj.iter() {
                                {
                                    let item_path = get_object_path(&parent_path, key);
                                    rsx! {
                                        div { key, class: "json-value-key-list__item",
                                            div { class: "json-value-key-list__key", title: "{item_path}", "{key}" }
                                            div { class: "json-value-key-list__value",
                                                JsonValue { value: value.clone(), parent_path: Some(item_path.clone()) }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                rsx! {
                    table { class: "json-value-table",
                        if obj.is_empty() {
                            div { class: "json-value-empty", "Empty Object" }
                        } else {
                            for (key , value) in obj.iter() {
                                {
                                    let item_path = get_object_path(&parent_path, key);
                                    rsx! {
                                        tr { key,
                                            th { title: "{item_path}", "{key}" }
                                            td {
                                                JsonValue { value: value.clone(), parent_path: Some(item_path.clone()) }
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

fn get_array_path(parent_path: &Option<String>, key: &str) -> String {
    format!("{}[{key}]", parent_path.as_ref().unwrap_or(&String::new()))
}

fn get_object_path(parent_path: &Option<String>, key: &str) -> String {
    format!("{}.{key}", parent_path.as_ref().unwrap_or(&String::new()))
}
