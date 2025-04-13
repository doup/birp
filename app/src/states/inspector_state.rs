use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct InspectorState {
    pub active: Signal<Option<u64>>,
    pub pinned: Signal<Vec<u64>>,
}

impl InspectorState {
    pub fn new() -> Self {
        Self {
            active: Signal::new(None),
            pinned: Signal::new(vec![]),
        }
    }
}
