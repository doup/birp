use client::Value;
use dioxus::prelude::*;

use crate::components::{Icon, JsonValue};
use crate::states::ConnectionState;
use crate::utils::get_short_type_name;

#[component]
pub fn ResourceInspector(resource_type: ReadOnlySignal<String>) -> Element {
    let client = use_context::<ConnectionState>().client;
    let update_signal = use_context::<ConnectionState>().update_signal;
    let mut resource = use_signal(|| None::<Value>);
    let update_fn = move || {
        async move {
            let res = client().get_resource(resource_type()).await;
            // TODO: Add proper error state
            resource.set(res.ok());
        }
    };

    // Update data when `resource_type` changes
    use_effect(move || {
        resource_type();
        spawn(update_fn());
    });

    // Update on refresh tick
    use_effect(move || {
        update_signal();
        spawn(update_fn());
    });

    rsx! {
        div { class: "inspector-card",
            div { class: "inspector-card__header-wrapper",
                div { class: "inspector-card__header",
                    div { class: "inspector-card__icon", {Icon::BookShelf.render()} }
                    div {
                        class: "inspector-card__name",
                        title: "{&resource_type}",
                        {get_short_type_name(&resource_type())}
                    }
                }
            }
            {
                match &*resource.read() {
                    Some(resource) => rsx! {
                        JsonValue { value: resource.clone() }
                    },
                    None => rsx! {},
                }
            }
        }
    }
}
