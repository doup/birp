use client::Entity;
use dioxus::prelude::*;

use crate::components::Icon;

#[component]
pub fn ValueEntity(entity: Entity) -> Element {
    rsx! {
        div { class: "value-entity",
            {Icon::Diamond.render()}
            "{entity}"
        }
    }
}
