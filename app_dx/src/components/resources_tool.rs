use dioxus::prelude::*;

use crate::states::ConnectionState;

#[component]
pub fn ResourcesTool() -> Element {
    let client = use_context::<ConnectionState>().client;
    let update_signal = use_context::<ConnectionState>().update_signal;
    let mut resources = use_signal(Vec::<String>::new);

    use_effect(move || {
        let _ = update_signal();

        spawn(async move {
            let res = client().list_resources().await;

            match res {
                Ok(res) => {
                    resources.set(res);
                }
                Err(_) => {
                    resources.set(Vec::new());
                }
            }
        });
    });

    rsx! {
        ul {
            for resource in resources.iter() {
                li { "{resource}" }
            }
        }
    }
}
