use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use dioxus::prelude::*;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use components::{Connection, EntitiesTool, Icon, ResourcesTool, TypesTool};
use states::{ConnectionState, EntitiesToolState, ResourcesToolState, TypesToolState};

mod bevy_type;
mod components;
mod states;
mod utils;

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
                    .with_min_inner_size(LogicalSize::new(530.0, 400.0))
                    .with_title("BiRP"),
            ),
        )
        .launch(App)
}

#[derive(PartialEq)]
enum Tool {
    Entities,
    Resources,
    Types,
}

#[component]
fn App() -> Element {
    use_context_provider(|| ConnectionState::new("http://127.0.0.1:15702"));
    use_context_provider(EntitiesToolState::new);
    use_context_provider(ResourcesToolState::new);
    use_context_provider(TypesToolState::new);

    let mut tool = use_signal(|| Tool::Entities);
    let tab_class = |tab: &Tool| {
        if *tool.read() == *tab {
            "tabs__item tabs__item--active"
        } else {
            "tabs__item"
        }
    };

    rsx! {
        document::Stylesheet { href: asset!("/assets/main.scss") }

        div { class: "layout",
            div { class: "layout__header",
                Connection {}
                div { class: "tabs",
                    div {
                        class: tab_class(&Tool::Entities),
                        onclick: move |_| tool.set(Tool::Entities),
                        {Icon::NodeTree.render()}
                        "Entities"
                    }
                    div {
                        class: tab_class(&Tool::Resources),
                        onclick: move |_| tool.set(Tool::Resources),
                        {Icon::BookShelf.render()}
                        "Resources"
                    }
                    div {
                        class: tab_class(&Tool::Types),
                        onclick: move |_| tool.set(Tool::Types),
                        {Icon::Squares.render()}
                        "Types"
                    }
                }
            }

            div { class: "layout__content",
                {
                    match *tool.read() {
                        Tool::Entities => rsx! {
                            EntitiesTool {}
                        },
                        Tool::Resources => rsx! {
                            ResourcesTool {}
                        },
                        Tool::Types => rsx! {
                            TypesTool {}
                        },
                    }
                }
            }
        }
    }
}
