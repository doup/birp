use dioxus::prelude::*;

#[component]
pub fn ValueBool(value: bool, onchange: EventHandler<bool>) -> Element {
    rsx! {
        input {
            r#type: "checkbox",
            checked: value,
            onchange: move |e| onchange.call(e.checked()),
        }
    }
}
