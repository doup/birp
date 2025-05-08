use dioxus::prelude::*;

use crate::{
    states::ConnectionState,
    utils::{add_zero_width_spaces, get_short_type_name},
};

#[component]
pub fn ResourcesTool() -> Element {
    let client = use_context::<ConnectionState>().client;
    let update_signal = use_context::<ConnectionState>().update_signal;
    let mut resources = use_signal(Vec::<String>::new);

    use_effect(move || {
        let _ = update_signal();

        spawn(async move {
            let res = client().list_resources().await;

            match res {
                Ok(res) => {
                    resources.set(res);
                }
                Err(_) => {
                    resources.set(Vec::new());
                }
            }
        });
    });

    rsx! {
        div { class: "sidebar-layout",
            div { class: "sidebar-layout__sidebar",
                div { class: "item-tree item-tree--root item-tree--flat",
                    for resource in resources.iter() {
                        div { class: "item-tree__item",
                            div { class: "item-tree__name",
                                {add_zero_width_spaces(&get_short_type_name(&resource))}
                            }
                        }
                    }
                }
            }
            div { class: "sidebar-layout__content" }
        }
    }
}
