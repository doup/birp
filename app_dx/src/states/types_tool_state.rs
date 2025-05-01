use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct TypesToolState {
    pub active: Signal<Option<String>>,
    pub filter: Signal<String>,
}

impl TypesToolState {
    pub fn new() -> Self {
        Self {
            active: Signal::new(None),
            filter: Signal::new(String::new()),
        }
    }
}
