use client::Value;
use dioxus::prelude::*;

#[component]
pub fn JsonValue(value: Value) -> Element {
    match value {
        Value::Null => rsx! {
            span { class: "json-value json-value--null", "Null" }
        },
        Value::Bool(b) => rsx! {
            span { class: "json-value json-value--bool",
                if b {
                    "True"
                } else {
                    "False"
                }
            }
        },
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
            table { class: "json-value-table",
                for item in arr.iter() {
                    tr {
                        th { "⚬" }
                        td {
                            JsonValue { value: item.clone() }
                        }
                    }
                }
            }
        },
        Value::Object(obj) => {
            let has_subobjects = obj
                .iter()
                .any(|(_, v)| matches!(v, Value::Object(_)) || matches!(v, Value::Array(_)));

            if has_subobjects {
                rsx! {
                    div { class: "json-value-key-list",
                        for (key , value) in obj.iter() {
                            div { class: "json-value-key-list__item",
                                div {
                                    class: "json-value-key-list__key",
                                    title: "{key}",
                                    "{key}"
                                }
                                div { class: "json-value-key-list__value",
                                    JsonValue { value: value.clone() }
                                }
                            }
                        }
                    }
                }
            } else {
                rsx! {
                    table { class: "json-value-table",
                        for (key , value) in obj.iter() {
                            tr {
                                th { title: "{key}", "{key}" }
                                td {
                                    JsonValue { value: value.clone() }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
