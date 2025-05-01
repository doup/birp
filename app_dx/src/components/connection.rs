use dioxus::{html::div, logger::tracing::info, prelude::*};
use std::time::Duration;

use crate::{
    components::{Icon, connection},
    states::ConnectionState,
};

#[component]
pub fn Connection() -> Element {
    let mut automatic_poll = use_context::<ConnectionState>().automatic_poll;
    let mut is_connected = use_context::<ConnectionState>().is_connected;
    let mut poll_interval = use_context::<ConnectionState>().poll_interval;
    let mut schema = use_context::<ConnectionState>().schema;
    let mut update_signal = use_context::<ConnectionState>().update_signal;
    let mut url = use_context::<ConnectionState>().url;
    let client = use_context::<ConnectionState>().client;
    let connection_status_class = use_memo(move || {
        format!(
            "connection__status {}",
            if is_connected() {
                "connection__status--connected"
            } else {
                "connection__status--disconnected"
            }
        )
    });
    let connection_polling_class = use_memo(move || {
        format!(
            "connection__polling polling {}",
            if automatic_poll() {
                "polling--auto"
            } else {
                ""
            }
        )
    });

    // Check if the client is connected
    use_coroutine(move |_rx: UnboundedReceiver<()>| async move {
        loop {
            let prev_is_connected = is_connected();
            let new_is_connected = match client().ping().await {
                Ok(_) => true,
                Err(_) => false,
            };

            // Load the schema each time we connect
            if !prev_is_connected && new_is_connected {
                match client().get_schema().await {
                    Ok(new_schema) => {
                        schema.set(new_schema);
                        is_connected.set(true)
                    }
                    Err(_) => is_connected.set(false),
                }
            } else {
                is_connected.set(new_is_connected);
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
        div { class: "connection",
            button {
                class: "button connection__refresh",
                onclick: move |_| { update_signal.set(()) },
                {Icon::Refresh.render()}
            }

            div { class: connection_status_class() }
            input {
                class: "connection__url",
                value: url(),
                oninput: move |e| url.set(e.data.value()),
            }

            div { class: connection_polling_class(),
                input {
                    class: "polling__checkbox",
                    id: "auto-poll",
                    r#type: "checkbox",
                    checked: automatic_poll(),
                    onchange: move |e| {
                        automatic_poll.set(e.data.checked());
                    },
                }
                label { class: "polling__label", r#for: "auto-poll", "Auto Refresh" }
                input {
                    class: "polling__range",
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
                span { class: "polling__value", "{poll_interval()}ms" }
            }
        }
    }
}
