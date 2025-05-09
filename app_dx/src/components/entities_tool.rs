use dioxus::prelude::*;

use crate::{
    components::{EntityInspector, HierarchyTree},
    states::EntitiesToolState,
};

#[component]
pub fn EntitiesTool() -> Element {
    let active_entity = use_context::<EntitiesToolState>().active;
    let pinned_entities = use_context::<EntitiesToolState>().pinned;

    // Merge pinned and active entities so we render the corresponding
    // inspectors in a single loop. By doing this we avoid weird scroll jumps
    // when pinning/unpinning entities.
    let entities = use_memo(move || {
        let mut entities = pinned_entities()
            .clone()
            .into_iter()
            .map(|id| (id, true))
            .collect::<Vec<_>>();

        if let Some(id) = active_entity() {
            if !pinned_entities().contains(&id) {
                entities.push((id, false));
            }
        }

        entities
    });

    rsx! {
        div { class: "sidebar-layout",
            div { class: "sidebar-layout__sidebar",
                HierarchyTree { level: 0, parent_id: None }
            }

            div { class: "sidebar-layout__content entities-grid",
                for (id , is_pinned) in entities().iter() {
                    EntityInspector { key: id, id: *id, is_pinned: *is_pinned }
                }
            }
        }
    }
}
