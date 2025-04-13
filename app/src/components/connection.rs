use dioxus::{logger::tracing::info, prelude::*};
use std::time::Duration;

use crate::states::ConnectionState;

#[component]
pub fn Connection() -> Element {
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
