use client::{Entity, EntityItem, EntityKind, SchemaKind, SchemaType};
use dioxus::prelude::*;

use crate::states::{ConnectionState, EntitiesToolState};

use crate::components::{ComponentInspector, Icon};
use crate::utils::get_short_type_name;

#[component]
pub fn EntityInspector(id: ReadOnlySignal<Entity>, is_pinned: bool) -> Element {
    let mut pinned = use_context::<EntitiesToolState>().pinned;
    let active = use_context::<EntitiesToolState>().active;
    let client = use_context::<ConnectionState>().client;
    let schema = use_context::<ConnectionState>().schema;
    let update_signal = use_context::<ConnectionState>().update_signal;

    let mut entity = use_signal(|| None::<EntityItem>);
    let update_fn = move || {
        async move {
            let res = client().get(id()).await;
            // TODO: Add proper error state
            entity.set(res.ok());
        }
    };
    let marker_components = use_memo(move || {
        entity.read().as_ref().map_or(vec![], |entity| {
            entity
                .components
                .iter()
                .filter(|(component, _)| {
                    // TODO: Improve this check, maybe also check for the
                    // `Value`? For example "Frustum" and "VisibleEntities" in a
                    // Camera are not markers. But right now are considered as such.
                    schema().get(component.as_str()).map_or(false, |s| {
                        s.kind == SchemaKind::Struct
                            && s.schema_type == SchemaType::Object
                            && s.additional_properties == Some(false)
                            && s.properties.is_empty()
                    })
                })
                .map(|(component, _)| component.clone())
                .collect()
        })
    });

    let class = format!(
        "inspector-card {}",
        if active() == Some(id()) {
            "inspector-card--active"
        } else {
            ""
        }
    );

    // Update data when `id` changes
    use_effect(move || {
        id();
        spawn(update_fn());
    });

    use_effect(move || {
        update_signal();
        spawn(update_fn());
    });

    match &*entity.read() {
        Some(entity) => rsx! {
            div { class,
                div { class: "inspector-card__header-wrapper",
                    div { class: "inspector-card__header",
                        div { class: "inspector-card__icon", {Icon::from(entity).render()} }
                        match entity.name() {
                            Some(name) => rsx! {
                                span { class: "inspector-card__name", "{name}" }
                            },
                            None => rsx! {
                                span { class: "inspector-card__name inspector-card__name--placeholder", "{EntityKind::from(entity):?}" }
                            },
                        }
                        span { class: "inspector-card__id", "{id}" }
                        div {
                            class: "inspector-card__pin",
                            onclick: move |_| {
                                pinned
                                    .with_mut(|pinned| {
                                        if pinned.contains(&id()) {
                                            pinned.retain(|&x| x != id());
                                        } else {
                                            pinned.push(id());
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

                if !marker_components().is_empty() {
                    div { class: "marker-components",
                        for component in marker_components() {
                            div { class: "marker-components__item", {get_short_type_name(&component)} }
                        }
                    }
                }


                for (component , value) in entity.components.iter() {
                    if !marker_components().contains(&component) {
                        ComponentInspector {
                            key: "{component}",
                            id: id(),
                            type_path: component.to_string(),
                            value: value.clone(),
                        }
                    }
                }
            }
        },
        None => rsx! {},
    }
}
