use client::get_root_entities;
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::logger::tracing::info;
use dioxus::prelude::*;

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
    let mut counter = use_signal(|| 0);
    let mut root = use_resource(move || async move { get_root_entities().await });

    rsx! {
        document::Stylesheet { href: asset!("/assets/main.scss") }
        Title {}
        div {
            "Counter: {counter}"

            button { onclick: move |_| counter.set(counter() - 1), "-" }
            button { onclick: move |_| counter.set(counter() + 1), "+" }

            ul {
                for i in 0..counter().max(0) {
                    li { "{i}" }
                }
            }

            hr {}

            button { onclick: move |_| root.restart(), "Reload" }

            ul {
                {
                    match &*root.read_unchecked() {
                        Some(Ok(res)) => rsx! {
                            table {
                                tr {
                                    th { "Kind" }
                                    th { "Name" }
                                    th { "ID" }
                                }
                            
                                tbody {
                                    for (idx , item) in res.result.iter().enumerate() {
                                        tr { class: "entity", onclick: move |_| info!("Clicked {}", idx),
                                            td { style: "font-size: 12px; min-width: 70px; display: inline-block;",
                                                "{item.kind():?}"
                                            }
                                            td { "{item.name()}" }
                                            td { "{item.entity}" }
                                        }
                                    }
                                }
                            }
                        },
                        Some(Err(_)) => rsx! {
                            div { "Loading root entities failed" }
                        },
                        None => rsx! {
                            div { "Loading root entities..." }
                        },
                    }
                }
            }

            pre { style: "width: 100%; text-wrap: wrap;", "root: {root.value():?}" }
        }
    }
}

#[component]
fn Title() -> Element {
    info!("Title Created");

    rsx! {
        div { id: "title",
            h1 { "🔎 Remote Inspector" }
        }
    }
}
