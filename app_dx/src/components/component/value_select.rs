use dioxus::prelude::*;

#[component]
pub fn ValueSelect(value: String, options: Vec<String>, onchange: EventHandler<String>) -> Element {
    let item_class = |is_selected: bool| {
        if is_selected {
            "options__item options__item--selected"
        } else {
            "options__item"
        }
    };

    rsx! {
        if options.len() > 3 {
            select {
                class: "value-select",
                onchange: move |e| {
                    onchange.call(e.value());
                },
                for option_value in options {
                    option {
                        value: "{option_value}",
                        selected: option_value == value,
                        "{option_value}"
                    }
                }
            }
        } else {
            div { class: "options",
                for option_value in options {
                    div {
                        class: item_class(option_value == value),
                        onclick: move |_| {
                            onchange.call(option_value.clone());
                        },
                        "{option_value}"
                    }
                }
            }
        }
    }
}
