use client::{Entity, EntityItem, EntityKind};
use dioxus::{logger::tracing::info, prelude::*};
use std::collections::BTreeMap;

use crate::{
    components::Icon,
    states::{ConnectionState, EntitiesToolState},
};

struct HierarchyItem {
    entity: EntityItem,
    expanded: bool,
}

#[component]
pub fn HierarchyTree(parent_id: Option<Entity>, level: u32) -> Element {
    let mut active_entity = use_context::<EntitiesToolState>().active;
    let pinned_entities = use_context::<EntitiesToolState>().pinned;
    let client = use_context::<ConnectionState>().client;
    let is_connected = use_context::<ConnectionState>().is_connected;
    let is_children = level > 0;
    let update_signal = use_context::<ConnectionState>().update_signal;
    let item_tree_class = format!(
        "item-tree {} {}",
        if is_connected() {
            "item-tree--connected"
        } else {
            "item-tree--disconnected"
        },
        if is_children {
            "item-tree--children"
        } else {
            "item-tree--root"
        }
    );

    let mut items: Signal<BTreeMap<Entity, HierarchyItem>> = use_signal(BTreeMap::new);

    let row_click = |id: Entity| {
        move |_: Event<MouseData>| {
            active_entity.set(Some(id));

            items.with_mut(|items| {
                let item = items.get_mut(&id).unwrap();

                if item.entity.children().is_empty() {
                    return;
                }

                item.expanded = !item.expanded;
            });
        }
    };

    use_effect(move || {
        update_signal();

        spawn(async move {
            let children = client().get_children(parent_id).await;

            match children {
                Ok(children) => {
                    let old_items = items.take();

                    items.set(
                        children
                            .into_iter()
                            .map(|entity| {
                                let expanded = old_items
                                    .get(&entity.id)
                                    .map(|item| item.expanded)
                                    .unwrap_or(false);

                                (entity.id, HierarchyItem { entity, expanded })
                            })
                            .collect::<_>(),
                    );
                }
                Err(e) => {
                    info!("Error fetching children: {}", e);
                }
            }
        });
    });

    rsx! {
        div { class: item_tree_class, style: "--item-tree-level: {level}",
            for (entity_id , item) in items.read().iter() {
                div {
                    key: "{entity_id}",
                    class: format!(
                        "item-tree__item {} {}",
                        if pinned_entities().contains(entity_id) {
                            "item-tree__item--pinned"
                        } else {
                            ""
                        },
                        if active_entity() == Some(*entity_id) { "item-tree__item--active" } else { "" },
                    ),
                    onclick: row_click(*entity_id),

                    div { class: "item-tree__chevron",
                        if !item.entity.children().is_empty() {
                            if item.expanded {
                                {Icon::ChevronDown.render()}
                            } else {
                                {Icon::ChevronRight.render()}
                            }
                        } else {
                            ""
                        }
                    }
                    div { class: "item-tree__icon kind-icon--{EntityKind::from(&item.entity):?}",
                        {Icon::from(&item.entity).render()}
                    }
                    match item.entity.name() {
                        Some(name) => rsx! {
                            div { class: "item-tree__name", "{name}" }
                        },
                        None => rsx! {
                            div { class: "item-tree__name item-tree__name--placeholder", "{EntityKind::from(&item.entity):?}" }
                        },
                    }
                    span { class: "item-tree__id", "{item.entity.id}" }
                }

                if item.expanded {
                    div { class: "item-tree__children",
                        HierarchyTree { level: level + 1, parent_id: Some(item.entity.id) }
                    }
                }
            }
        }
    }
}
