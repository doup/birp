use dioxus::prelude::*;

#[component]
pub fn ValueString(value: String, onchange: EventHandler<String>) -> Element {
    rsx! {
        input {
            class: "value-string",
            r#type: "text",
            value,
            oninput: move |e| {
                onchange.call(e.value());
            },
        }
    }
}
