use std::collections::BTreeMap;
use std::time::Duration;

use client::{BrpClient, HierarchyEntity};
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::logger::tracing::info;
use dioxus::prelude::*;
use icon::Icon;

mod icon;

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

#[derive(Clone, Copy)]
struct ConnectionState {
    automatic_poll: Signal<bool>,
    client: Memo<BrpClient>,
    is_connected: Signal<bool>,
    poll_interval: Signal<u64>,
    update_signal: Signal<()>,
    url: Signal<String>,
}

impl ConnectionState {
    fn new(url: impl Into<String>) -> Self {
        let automatic_poll = Signal::new(true);
        let is_connected = Signal::new(true);
        let poll_interval = Signal::new(500_u64);
        let update_signal = Signal::new(());
        let url = Signal::new(url.into());

        let client = Memo::new(move || BrpClient::new(url()));

        Self {
            automatic_poll,
            client,
            is_connected,
            poll_interval,
            update_signal,
            url,
        }
    }
}

#[derive(Clone, Copy)]
struct InspectorState {
    active: Signal<Option<u64>>,
    pinned: Signal<Vec<u64>>,
}

impl InspectorState {
    fn new() -> Self {
        Self {
            active: Signal::new(None),
            pinned: Signal::new(vec![]),
        }
    }
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

#[component]
fn Connection() -> Element {
    let mut automatic_poll = use_context::<ConnectionState>().automatic_poll;
    let mut is_connected = use_context::<ConnectionState>().is_connected;
    let mut poll_interval = use_context::<ConnectionState>().poll_interval;
    let mut update_signal = use_context::<ConnectionState>().update_signal;
    let mut url = use_context::<ConnectionState>().url;
    let client = use_context::<ConnectionState>().client;
    let connection_indicator_class = use_memo(move || {
        format!(
            "connection-indicator {}",
            if is_connected() {
                "connection-indicator--connected"
            } else {
                "connection-indicator--disconnected"
            }
        )
    });

    // Check if the client is connected
    use_coroutine(move |_rx: UnboundedReceiver<()>| async move {
        loop {
            match client().ping().await {
                Ok(_) => is_connected.set(true),
                Err(_) => is_connected.set(false),
            }

            tokio::time::sleep(Duration::from_millis(1000)).await;
        }
    });

    // Notify components to update
    use_coroutine(move |_rx: UnboundedReceiver<()>| async move {
        loop {
            if automatic_poll() && is_connected() {
                update_signal.set(());
            }

            tokio::time::sleep(Duration::from_millis(poll_interval())).await;
        }
    });

    rsx! {
        div {
            div { class: connection_indicator_class() }
            input {
                value: url(),
                oninput: move |e| {
                    let new_url = e.data.value();
                    url.set(new_url);
                },
            }
        }
        input {
            id: "auto-poll",
            r#type: "checkbox",
            checked: automatic_poll(),
            onchange: move |e| {
                automatic_poll.set(e.data.checked());
            },
        }
        label { r#for: "auto-poll", "Automatic Poll" }
        div {
            "Poll Interval:"
            input {
                r#type: "range",
                value: "2",
                min: "0",
                max: "4",
                oninput: move |e| {
                    let slider_value = e.data.value().parse::<u32>().unwrap_or(0);
                    let ms = 125 * 2u64.pow(slider_value);
                    poll_interval.set(ms);
                },
                onchange: move |e| {
                    info!("final slider value: {}", e.data.value());
                },
            }
            "{poll_interval()}ms"
        }
        button { onclick: move |_| { update_signal.set(()) }, "Refresh" }
    }
}

struct HierarchyItem {
    entity: HierarchyEntity,
    expanded: bool,
}

#[component]
fn HierarchyTree(parent_id: Option<u64>, level: u32) -> Element {
    let mut active_entity = use_context::<InspectorState>().active;
    let client = use_context::<ConnectionState>().client;
    let is_connected = use_context::<ConnectionState>().is_connected;
    let is_children = level > 0;
    let update_signal = use_context::<ConnectionState>().update_signal;
    let entity_tree_class = format!(
        "entity-tree {} {}",
        if is_connected() {
            "entity-tree--connected"
        } else {
            "entity-tree--disconnected"
        },
        if is_children {
            "entity-tree--children"
        } else {
            ""
        }
    );

    let mut items: Signal<BTreeMap<u64, HierarchyItem>> = use_signal(BTreeMap::new);

    let row_click = |id: u64| {
        move |_: Event<MouseData>| {
            active_entity.set(Some(id));

            items.with_mut(|items| {
                let item = items.get_mut(&id).unwrap();

                if item.entity.children().len() == 0 {
                    return;
                }

                item.expanded = !item.expanded;
            });
        }
    };

    use_effect(move || {
        let _ = update_signal();

        spawn(async move {
            let children = client().get_children(parent_id).await;

            match children {
                Ok(children) => {
                    let old_items = items.take();

                    items.set(
                        children
                            .into_iter()
                            .map(|entity| {
                                let expanded = old_items
                                    .get(&entity.id)
                                    .map(|item| item.expanded)
                                    .unwrap_or(false);

                                (entity.id, HierarchyItem { entity, expanded })
                            })
                            .collect::<_>(),
                    );
                }
                Err(e) => {
                    info!("Error fetching children: {}", e);
                }
            }
        });
    });

    rsx! {
        div { class: entity_tree_class, style: "--entity-tree-level: {level}",
            for (entity_id , item) in items.read().iter() {
                div {
                    key: "{entity_id}",
                    class: format!(
                        "entity-tree__item {}",
                        if active_entity() == Some(*entity_id) {
                            "entity-tree__item--active"
                        } else {
                            ""
                        },
                    ),
                    onclick: row_click(*entity_id),

                    div { class: "entity-tree__chevron",
                        if item.entity.children().len() > 0 {
                            if item.expanded {
                                {Icon::ChevronDown.render()}
                            } else {
                                {Icon::ChevronRight.render()}
                            }
                        } else {
                            ""
                        }
                    }
                    div { class: "entity-tree__kind", {Icon::from(item.entity.kind()).render()} }
                    div { class: "entity-tree__name", "{item.entity.name()}" }
                    span { class: "entity-tree__id", "{item.entity.id}" }
                }

                if item.expanded {
                    div { class: "entity-tree__children",
                        HierarchyTree { level: level + 1, parent_id: Some(item.entity.id) }
                    }
                }
            }
        }
    }
}
