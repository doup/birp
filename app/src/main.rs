use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use components::{Connection, EntitiesTool};
use states::{ConnectionState, EntitiesToolState};

mod components;
mod states;

fn main() {
    // Initialize tracing with filter
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            EnvFilter::builder()
                .parse("warn,bevy_remote=error,client=debug,app=debug")
                .unwrap(),
        )
        .init();

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

enum Tool {
    Entities,
    Resources,
}

#[component]
fn App() -> Element {
    use_context_provider(|| ConnectionState::new("http://127.0.0.1:15702"));
    use_context_provider(|| EntitiesToolState::new());
    let mut tool = use_signal(|| Tool::Entities);

    rsx! {
        document::Stylesheet { href: asset!("/assets/main.scss") }
        Connection {}

        div { class: "tabs",
            div {
                class: "tabs__item",
                onclick: move |_| tool.set(Tool::Entities),
                "Entities"
            }
            div {
                class: "tabs__item",
                onclick: move |_| tool.set(Tool::Resources),
                "Resources"
            }
        }

        {
            match *tool.read() {
                Tool::Entities => rsx! {
                    EntitiesTool {}
                },
                Tool::Resources => rsx! {
                    div { "Resources" }
                },
            }
        }
    }
}
