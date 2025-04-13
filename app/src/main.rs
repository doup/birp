use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;

use components::{Connection, HierarchyTree};
use states::{ConnectionState, InspectorState};

mod components;
mod states;

fn main() {
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            Config::new().with_window(
                WindowBuilder::new()
                    .with_resizable(true)
                    .with_title("Remote Inspector"),
            ),
        )
        .launch(App)
}

#[component]
fn App() -> Element {
    use_context_provider(|| ConnectionState::new("http://127.0.0.1:15702"));
    use_context_provider(|| InspectorState::new());

    let active_entity = use_context::<InspectorState>().active;

    rsx! {
        document::Stylesheet { href: asset!("/assets/main.scss") }
        Connection {}
        div {
            hr {}

            div { class: "module",
                HierarchyTree { level: 0, parent_id: None }
            }

            {format!("{:?}", active_entity())}
        }
    }
}
