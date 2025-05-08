use dioxus::prelude::*;

use crate::{
    components::{EntityInspector, HierarchyTree},
    states::EntitiesToolState,
};

#[component]
pub fn EntitiesTool() -> Element {
    let active_entity = use_context::<EntitiesToolState>().active;
    let pinned_entities = use_context::<EntitiesToolState>().pinned;

    rsx! {
        div { class: "sidebar-layout",
            div { class: "sidebar-layout__sidebar",
                HierarchyTree { level: 0, parent_id: None }
            }

            div { class: "sidebar-layout__content entities-grid",
                for id in pinned_entities().iter() {
                    EntityInspector { id: *id, is_pinned: true }
                }

                if let Some(id) = active_entity() {
                    if !pinned_entities().contains(&id) {
                        EntityInspector { id, is_pinned: false }
                    }
                }
            }
        }
    }
}
