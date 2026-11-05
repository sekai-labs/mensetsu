use dioxus::prelude::*;
use crate::services::{api::ApiService, endpoint_config::{set_persisted_server_url, validate_and_normalize_url}};
use crate::ui::test_ids::test_ids;

#[derive(Props, Clone, PartialEq)]
pub struct ServerSetupProps {
    pub api: ApiService,
    pub current_url: Option<String>,
    pub on_saved: EventHandler<String>,
    pub on_cancel: Option<EventHandler<()>>,
}

#[component]
pub fn ServerSetupView(props: ServerSetupProps) -> Element {
    let mut url_text = use_signal(|| props.current_url.clone().unwrap_or_default());
    let mut error_msg = use_signal(|| Option::<String>::None);
    let mut is_submitting = use_signal(|| false);

    let handle_submit = move |_| {
        let input = url_text.read().clone();
        match validate_and_normalize_url(&input) {
            Err(e) => {
                error_msg.set(Some(e.to_string()));
            }
            Ok(normalized) => {
                error_msg.set(None);
                is_submitting.set(true);
                let api = props.api.clone();
                let on_saved = props.on_saved;
                spawn(async move {
                    if let Err(e) = api.set_base_url(normalized.clone()).await {
                        error_msg.set(Some(e.to_string()));
                        is_submitting.set(false);
                        return;
                    }
                    if let Err(e) = set_persisted_server_url(&normalized) {
                        error_msg.set(Some(e.to_string()));
                        is_submitting.set(false);
                        return;
                    }
                    is_submitting.set(false);
                    on_saved.call(normalized);
                });
            }
        }
    };

    rsx! {
        div {
            class: "center-container",
            id: "{test_ids::SERVER_SETUP_SCREEN}",
            "data-testid": "{test_ids::SERVER_SETUP_SCREEN}",
            div {
                class: "card",
                h1 { class: "title-1", "Connect to Mensetsu Server" }
                p {
                    class: "dim-label",
                    style: "margin-bottom: 20px;",
                    "Enter your self-hosted Rust API base URL (e.g. https://api.example.com or http://127.0.0.1:3000 for local development)."
                }

                if let Some(err) = error_msg.read().as_ref() {
                    div {
                        class: "error-banner",
                        id: "{test_ids::SERVER_URL_ERROR_MESSAGE}",
                        "data-testid": "{test_ids::SERVER_URL_ERROR_MESSAGE}",
                        "{err}"
                    }
                }

                div { style: "margin-bottom: 16px;",
                    label { class: "dim-label", style: "display: block; margin-bottom: 6px;", "API Base URL" }
                    input {
                        id: "{test_ids::INPUT_SERVER_URL}",
                        "data-testid": "{test_ids::INPUT_SERVER_URL}",
                        r#type: "url",
                        placeholder: "https://api.example.com",
                        value: "{url_text}",
                        oninput: move |evt| url_text.set(evt.value().clone()),
                    }
                }

                div { style: "display: flex; gap: 12px; justify-content: flex-end; margin-top: 24px;",
                    if let Some(on_cancel) = props.on_cancel {
                        button {
                            class: "secondary",
                            onclick: move |_| on_cancel.call(()),
                            "Cancel"
                        }
                    }
                    button {
                        class: "primary",
                        id: "{test_ids::BUTTON_SAVE_SERVER_URL}",
                        "data-testid": "{test_ids::BUTTON_SAVE_SERVER_URL}",
                        disabled: *is_submitting.read(),
                        onclick: handle_submit,
                        if *is_submitting.read() { "Connecting..." } else { "Connect" }
                    }
                }
            }
        }
    }
}
