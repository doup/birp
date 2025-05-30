use client::SchemaType;
use dioxus::prelude::*;

#[component]
pub fn ValueNumber(value: String, onchange: EventHandler<f64>, schema_type: SchemaType) -> Element {
    let (min_attr, step_attr) = match schema_type {
        SchemaType::Float => (None, Some("any".to_string())),
        SchemaType::Int => (None, Some("1".to_string())),
        SchemaType::Uint => (Some("0".to_string()), Some("1".to_string())),
        _ => (None, None),
    };

    rsx! {
        input {
            r#type: "number",
            value,
            min: min_attr,
            step: step_attr,
            oninput: move |e| {
                if let Ok(number) = e.value().parse::<f64>() {
                    onchange.call(number);
                }
            },
        }
    }
}
