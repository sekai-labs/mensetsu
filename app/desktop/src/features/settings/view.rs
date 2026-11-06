use dioxus::prelude::*;
use mensetsu_contracts::models::{LlmConfigItemDto, UpsertLlmConfigRequestDto};
use crate::services::{api::ApiService, endpoint_config::{set_persisted_server_url, validate_and_normalize_url}};
use crate::ui::test_ids::test_ids;

#[derive(Props, Clone, PartialEq)]
pub struct SettingsProps {
    pub api: ApiService,
    pub on_back: EventHandler<()>,
    pub on_server_changed: EventHandler<()>,
}

#[component]
pub fn SettingsView(props: SettingsProps) -> Element {
    let mut server_url_input = use_signal(|| props.api.get_base_url());
    let mut server_error = use_signal(|| Option::<String>::None);
    let mut is_saving_server = use_signal(|| false);

    let mut llm_settings = use_signal(Vec::<LlmConfigItemDto>::new);
    let mut new_provider = use_signal(String::new);
    let mut new_api_key = use_signal(String::new);
    let mut llm_error = use_signal(|| Option::<String>::None);
    let mut is_saving_llm = use_signal(|| false);

    let api = props.api.clone();
    let on_back = props.on_back;
    let on_server_changed = props.on_server_changed;

    let load_llm = {
        let api = api.clone();
        move || {
            let api = api.clone();
            spawn(async move {
                if let Ok(settings) = api.get_llm_settings().await {
                    llm_settings.set(settings);
                }
            });
        }
    };
    use_effect({
        let load_fn = load_llm.clone();
        move || {
            load_fn();
        }
    });

    let handle_change_server = {
        let api = api.clone();
        move |_| {
            let raw = server_url_input.read().clone();
            match validate_and_normalize_url(&raw) {
                Err(e) => {
                    server_error.set(Some(e.to_string()));
                }
                Ok(normalized) => {
                    server_error.set(None);
                    is_saving_server.set(true);
                    let api = api.clone();

                    spawn(async move {
                        if let Err(e) = api.set_base_url(normalized.clone()).await {
                            server_error.set(Some(e.to_string()));
                            is_saving_server.set(false);
                            return;
                        }
                        if let Err(e) = set_persisted_server_url(&normalized) {
                            server_error.set(Some(e.to_string()));
                            is_saving_server.set(false);
                            return;
                        }
                        is_saving_server.set(false);
                        on_server_changed.call(());
                    });
                }
            }
        }
    };

    let handle_save_llm = {
        let api = api.clone();
        move |_| {
            let provider = new_provider.read().trim().to_string();
            let key = new_api_key.read().trim().to_string();

            if provider.is_empty() {
                llm_error.set(Some("Provider name is required".to_string()));
                return;
            }
            if key.is_empty() {
                llm_error.set(Some("API key is required".to_string()));
                return;
            }

            llm_error.set(None);
            is_saving_llm.set(true);
            let api = api.clone();
            let load_fn = load_llm.clone();

            spawn(async move {
                let mut req = UpsertLlmConfigRequestDto::new(provider);
                req.api_key = Some(Some(key));
                match api.update_llm_setting(req).await {
                    Ok(_) => {
                        new_provider.set(String::new());
                        new_api_key.set(String::new());
                        load_fn();
                    }
                    Err(e) => {
                        llm_error.set(Some(e.to_string()));
                    }
                }
                is_saving_llm.set(false);
            });
        }
    };

    rsx! {
        div {
            class: "app-layout",
            id: "{test_ids::SETTINGS_VIEW}",
            "data-testid": "{test_ids::SETTINGS_VIEW}",

            header { class: "app-header",
                div { style: "display: flex; align-items: center; gap: 16px;",
                    button {
                        class: "secondary",
                        id: "{test_ids::SETTINGS_BACK_BUTTON}",
                        "data-testid": "{test_ids::SETTINGS_BACK_BUTTON}",
                        onclick: move |_| on_back.call(()),
                        "← Back"
                    }
                    h1 { class: "title-2", "Settings" }
                }
            }

            main { class: "app-content",
                div { style: "max-width: 680px; display: flex; flex-direction: column; gap: 32px;",

                    div { class: "card", style: "max-width: 100%;",
                        h2 { class: "title-2", style: "margin-bottom: 8px;", "Rust API Server Endpoint" }
                        p {
                            class: "dim-label",
                            style: "margin-bottom: 16px;",
                            "Changing the server resets your active session and credentials so tokens do not leak."
                        }

                        if let Some(err) = server_error.read().as_ref() {
                            div {
                                class: "error-banner",
                                id: "{test_ids::SETTINGS_SERVER_URL_ERROR_MESSAGE}",
                                "data-testid": "{test_ids::SETTINGS_SERVER_URL_ERROR_MESSAGE}",
                                "{err}"
                            }
                        }

                        div { style: "margin-bottom: 16px;",
                            label { class: "dim-label", style: "display: block; margin-bottom: 4px;", "API Base URL" }
                            input {
                                id: "{test_ids::SETTINGS_INPUT_SERVER_URL}",
                                "data-testid": "{test_ids::SETTINGS_INPUT_SERVER_URL}",
                                placeholder: "https://api.example.com",
                                value: "{server_url_input}",
                                oninput: move |evt| server_url_input.set(evt.value().clone()),
                            }
                        }

                        div { style: "display: flex; justify-content: flex-end;",
                            button {
                                class: "primary",
                                id: "{test_ids::SETTINGS_BUTTON_SAVE_SERVER_URL}",
                                "data-testid": "{test_ids::SETTINGS_BUTTON_SAVE_SERVER_URL}",
                                disabled: *is_saving_server.read(),
                                onclick: handle_change_server,
                                if *is_saving_server.read() { "Updating..." } else { "Change Server" }
                            }
                        }
                    }


                    div { class: "card", style: "max-width: 100%;",
                        h2 { class: "title-2", style: "margin-bottom: 8px;", "LLM Provider Configuration" }
                        p { class: "dim-label", style: "margin-bottom: 16px;", "Configure provider API keys for intelligence analysis." }

                        if let Some(err) = llm_error.read().as_ref() {
                            div { class: "error-banner", "{err}" }
                        }

                        div { style: "display: flex; flex-direction: column; gap: 12px; margin-bottom: 16px;",
                            div {
                                label { class: "dim-label", style: "display: block; margin-bottom: 4px;", "Provider" }
                                input {
                                    id: "{test_ids::SETTINGS_PROVIDER_INPUT}",
                                    "data-testid": "{test_ids::SETTINGS_PROVIDER_INPUT}",
                                    placeholder: "e.g. openai, anthropic, openrouter",
                                    value: "{new_provider}",
                                    oninput: move |evt| new_provider.set(evt.value().clone()),
                                }
                            }
                            div {
                                label { class: "dim-label", style: "display: block; margin-bottom: 4px;", "API Key" }
                                input {
                                    id: "{test_ids::SETTINGS_KEY_INPUT}",
                                    "data-testid": "{test_ids::SETTINGS_KEY_INPUT}",
                                    r#type: "password",
                                    placeholder: "sk-...",
                                    value: "{new_api_key}",
                                    oninput: move |evt| new_api_key.set(evt.value().clone()),
                                }
                            }
                        }

                        div { style: "display: flex; justify-content: flex-end; margin-bottom: 24px;",
                            button {
                                class: "primary",
                                id: "{test_ids::SETTINGS_SAVE_BUTTON}",
                                "data-testid": "{test_ids::SETTINGS_SAVE_BUTTON}",
                                disabled: *is_saving_llm.read(),
                                onclick: handle_save_llm,
                                if *is_saving_llm.read() { "Saving..." } else { "Save Key" }
                            }
                        }

                        h3 { style: "margin-bottom: 8px;", "Configured Providers" }
                        div {
                            id: "{test_ids::SETTINGS_LIST}",
                            "data-testid": "{test_ids::SETTINGS_LIST}",
                            class: "list-box",
                            if llm_settings.read().is_empty() {
                                p { class: "dim-label", "No external provider keys configured yet." }
                            } else {
                                for item in llm_settings.read().iter() {
                                    div { class: "list-item", key: "{item.provider}",
                                        div {
                                            strong { style: "display: block;", "{item.provider}" }
                                            span { class: "dim-label", "Configured (key masked)" }
                                        }
                                        span { style: "color: var(--success); font-weight: 500;", "Active ✓" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
