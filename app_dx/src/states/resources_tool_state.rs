use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct ResourcesToolState {
    pub active: Signal<Option<String>>,
}

impl ResourcesToolState {
    pub fn new() -> Self {
        Self {
            active: Signal::new(None),
        }
    }
}
