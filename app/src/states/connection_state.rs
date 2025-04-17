use client::BrpClient;
use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct ConnectionState {
    pub automatic_poll: Signal<bool>,
    pub client: Memo<BrpClient>,
    pub is_connected: Signal<bool>,
    pub poll_interval: Signal<u64>,
    /// Signal to notify components to update
    pub update_signal: Signal<()>,
    pub url: Signal<String>,
}

impl ConnectionState {
    pub fn new(url: impl Into<String>) -> Self {
        let automatic_poll = Signal::new(true);
        let is_connected = Signal::new(false);
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
