use dioxus::prelude::*;

use crate::{
    components::{EntityInspector, HierarchyTree},
    states::InspectorState,
};

#[component]
pub fn EntitiesTool() -> Element {
    let active_entity = use_context::<InspectorState>().active;
    let pinned_entities = use_context::<InspectorState>().pinned;

    rsx! {
        div { class: "entities",
            div { class: "card entities__hierarchy",
                HierarchyTree { level: 0, parent_id: None }
            }

            div { class: "entities__details",
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
