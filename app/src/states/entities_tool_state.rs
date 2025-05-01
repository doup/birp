use client::Entity;
use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct EntitiesToolState {
    pub active: Signal<Option<Entity>>,
    pub pinned: Signal<Vec<Entity>>,
}

impl EntitiesToolState {
    pub fn new() -> Self {
        Self {
            active: Signal::new(None),
            pinned: Signal::new(vec![]),
        }
    }
}
