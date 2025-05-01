use client::{Entity, EntityItem};
use dioxus::prelude::*;

use crate::states::{ConnectionState, EntitiesToolState};

#[component]
pub fn EntityInspector(id: Entity, is_pinned: bool) -> Element {
    let mut pinned = use_context::<EntitiesToolState>().pinned;
    let active = use_context::<EntitiesToolState>().active;
    let client = use_context::<ConnectionState>().client;
    let update_signal = use_context::<ConnectionState>().update_signal;

    let mut entity = use_signal(|| None::<EntityItem>);
    let class = format!(
        "entity-inspector {} card",
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

    rsx! {
        div { class,
            {
                match &*entity.read() {
                    Some(entity) => rsx! {
                        {format!("{}", entity.name().unwrap_or(format!("{}", id)))}
                    },
                    None => rsx! {},
                }
            }

            button {
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
                    "Unpin"
                } else {
                    "Pin"
                }
            }
        }
    }
}
