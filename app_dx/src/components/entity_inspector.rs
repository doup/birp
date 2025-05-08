use client::{Entity, EntityItem, EntityKind};
use dioxus::prelude::*;

use crate::states::{ConnectionState, EntitiesToolState};

use crate::components::{ComponentInspector, Icon};

#[component]
pub fn EntityInspector(id: Entity, is_pinned: bool) -> Element {
    let mut pinned = use_context::<EntitiesToolState>().pinned;
    let active = use_context::<EntitiesToolState>().active;
    let client = use_context::<ConnectionState>().client;
    let update_signal = use_context::<ConnectionState>().update_signal;

    let mut entity = use_signal(|| None::<EntityItem>);
    let class = format!(
        "entity-inspector {}",
        if active() == Some(id) {
            "entity-inspector--active"
        } else {
            ""
        }
    );

    use_effect(move || {
        let _ = update_signal();

        spawn(async move {
            let res = client().get(id).await;
            // TODO: Add proper error state
            entity.set(res.ok());
        });
    });

    match &*entity.read() {
        Some(entity) => rsx! {
            div { class,
                div { class: "entity-inspector__header-wrapper",
                    div { class: "entity-inspector__header",
                        div { class: "entity-inspector__kind-icon", {Icon::from(entity).render()} }
                        match entity.name() {
                            Some(name) => rsx! {
                                span { class: "entity-inspector__name", "{name}" }
                            },
                            None => rsx! {
                                span { class: "entity-inspector__name entity-inspector__name--placeholder",
                                    "{EntityKind::from(entity):?}"
                                }
                            },
                        }
                        span { class: "entity-inspector__id", "{id}" }
                        div {
                            class: "entity-inspector__pin",
                            onclick: move |_| {
                                pinned
                                    .with_mut(|pinned| {
                                        if pinned.contains(&id) {
                                            pinned.retain(|&x| x != id);
                                        } else {
                                            pinned.push(id);
                                        }
                                    });
                            },

                            if is_pinned {
                                {Icon::Unpin.render()}
                            } else {
                                {Icon::Pin.render()}
                            }
                        }
                    }
                }

                for (component , value) in entity.components.iter() {
                    ComponentInspector {
                        type_path: component.to_string(),
                        value: value.clone(),
                    }
                }
            }
        },
        None => rsx! {},
    }
}
