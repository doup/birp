use dioxus::prelude::*;

use crate::{
    components::ResourceInspector,
    states::{ConnectionState, ResourcesToolState},
    utils::{add_zero_width_spaces, get_short_type_name},
};

#[component]
pub fn ResourcesTool() -> Element {
    let client = use_context::<ConnectionState>().client;
    let update_signal = use_context::<ConnectionState>().update_signal;
    let mut active = use_context::<ResourcesToolState>().active;
    let mut resources = use_signal(Vec::<String>::new);
    let row_click = |ty: String| {
        move |_| {
            active.set(Some(ty.clone()));
        }
    };

    use_effect(move || {
        update_signal();

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
                    for res in resources.iter() {
                        div {
                            class: format!(
                                "item-tree__item {}",
                                if active().as_ref() == Some(&res) { "item-tree__item--active" } else { "" },
                            ),
                            onclick: row_click(res.clone()),
                            div { class: "item-tree__name",
                                {add_zero_width_spaces(&get_short_type_name(&res))}
                            }
                        }
                    }
                }
            }
            div { class: "sidebar-layout__content",
                if let Some(active_res) = active() {
                    ResourceInspector { resource_type: active_res.clone() }
                }
            }
        }
    }
}
