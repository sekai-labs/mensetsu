use dioxus::prelude::*;
use mensetsu_contracts::models::{
    LlmConfigItemDto, TokenUsageCursorDto, TokenUsageResponseDto, TokenUsageRowDto,
    UpsertLlmConfigRequestDto,
};
use crate::services::{api::ApiService, endpoint_config::{set_persisted_server_url, validate_and_normalize_url}};
use crate::ui::test_ids::test_ids;

fn format_token_count(value: i64) -> String {
    let digits = value.unsigned_abs().to_string();
    let mut formatted = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, ch) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            formatted.push(',');
        }
        formatted.push(ch);
    }
    if value < 0 {
        format!("-{formatted}")
    } else {
        formatted
    }
}

fn format_usage_time(timestamp: &str) -> String {
    let mut formatted = timestamp.chars().take(16).collect::<String>().replace('T', " ");
    formatted.push_str(" UTC");
    formatted
}

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
    let mut token_usage = use_signal(|| Option::<TokenUsageResponseDto>::None);
    let mut token_rows = use_signal(Vec::<TokenUsageRowDto>::new);
    let mut token_cursor = use_signal(|| Option::<TokenUsageCursorDto>::None);
    let mut token_error = use_signal(|| Option::<String>::None);
    let mut token_loading = use_signal(|| false);
    let mut token_loading_older = use_signal(|| false);
    let mut token_older_error = use_signal(|| false);

    let mut load_usage = {
        let api = props.api.clone();
        move || {
            if *token_loading.read() || *token_loading_older.read() {
                return;
            }
            token_loading.set(true);
            token_error.set(None);
            token_older_error.set(false);
            token_rows.set(Vec::new());
            let api = api.clone();
            spawn(async move {
                match api.get_token_usage(None).await {
                    Ok(response) => {
                        token_rows.set(response.items.clone());
                        token_cursor.set(response.next_cursor.clone().flatten().map(|cursor| *cursor));
                        token_usage.set(Some(response));
                    }
                    Err(error) => token_error.set(Some(error.to_string())),
                }
                token_loading.set(false);
            });
        }
    };

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
        let mut load_usage_fn = load_usage.clone();
        move || {
            load_fn();
            load_usage_fn();
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
    let load_older_usage = {
        let api = props.api.clone();
        move |_| {
            let cursor = token_cursor.read().clone();
            if cursor.is_none() || *token_loading.read() || *token_loading_older.read() {
                return;
            }
            token_loading_older.set(true);
            token_older_error.set(false);
            let api = api.clone();
            spawn(async move {
                match api.get_token_usage(cursor).await {
                    Ok(response) => {
                        token_rows.write().extend(response.items);
                        token_cursor.set(response.next_cursor.flatten().map(|cursor| *cursor));
                    }
                    Err(_) => token_older_error.set(true),
                }
                token_loading_older.set(false);
            });
        }
    };

    rsx! {
        div {
            class: "app-layout",
            id: "{test_ids::SETTINGS_VIEW}",
            "data-testid": "{test_ids::SETTINGS_VIEW}",

            header { class: "app-header",
                div { class: "settings-header-left",
                    button {
                        class: "secondary",
                        id: "{test_ids::SETTINGS_BACK_BUTTON}",
                        "data-testid": "{test_ids::SETTINGS_BACK_BUTTON}",
                        onclick: move |_| on_back.call(()),
                        "← Back"
                    }
                    div { class: "settings-header-title",
                        h1 { class: "title-2 settings-page-title", "Account settings" }
                        p { class: "dim-label", "Connection, provider credentials, and token usage" }
                    }
                }
            }

            main { class: "app-content",
                div { class: "settings-shell",
                    nav { class: "settings-nav", "aria-label": "Settings sections",
                        span { class: "settings-nav-label", "Settings" }
                        a { class: "settings-nav-link", href: "#settings-providers", "Providers" }
                        a { class: "settings-nav-link", href: "#settings-usage", "Token usage" }
                        a { class: "settings-nav-link", href: "#settings-server", "Server" }
                    }

                    div { class: "settings-sections",
                        section {
                            id: "settings-providers",
                            class: "settings-card",
                            "aria-labelledby": "settings-provider-title",

                            div { class: "settings-card-head",
                                div {
                                    h2 { id: "settings-provider-title", class: "settings-title", "Provider credentials" }
                                    p { class: "settings-copy", "Add a provider key for model requests. Saved keys are never shown here." }
                                }
                            }

                            if let Some(err) = llm_error.read().as_ref() {
                                div { class: "error-banner", role: "alert", "{err}" }
                            }

                            div { class: "settings-form-grid",
                                div {
                                    label { class: "field-label", r#for: "{test_ids::SETTINGS_PROVIDER_INPUT}", "Provider" }
                                    input {
                                        id: "{test_ids::SETTINGS_PROVIDER_INPUT}",
                                        "data-testid": "{test_ids::SETTINGS_PROVIDER_INPUT}",
                                        placeholder: "e.g. openai, anthropic",
                                        value: "{new_provider}",
                                        oninput: move |evt| new_provider.set(evt.value().clone()),
                                    }
                                }
                                div {
                                    label { class: "field-label", r#for: "{test_ids::SETTINGS_KEY_INPUT}", "API key" }
                                    input {
                                        id: "{test_ids::SETTINGS_KEY_INPUT}",
                                        "data-testid": "{test_ids::SETTINGS_KEY_INPUT}",
                                        r#type: "password",
                                        autocomplete: "new-password",
                                        placeholder: "Enter provider key",
                                        value: "{new_api_key}",
                                        oninput: move |evt| new_api_key.set(evt.value().clone()),
                                    }
                                }
                            }
                            p { class: "field-help", "Keys are stored securely by the server. The saved value is never displayed in this app." }

                            div { class: "settings-actions",
                                button {
                                    class: "primary",
                                    id: "{test_ids::SETTINGS_SAVE_BUTTON}",
                                    "data-testid": "{test_ids::SETTINGS_SAVE_BUTTON}",
                                    disabled: *is_saving_llm.read(),
                                    onclick: handle_save_llm,
                                    if *is_saving_llm.read() { "Saving…" } else { "Save key" }
                                }
                            }

                            div { class: "settings-subsection",
                                div { class: "settings-subsection-head",
                                    h3 { class: "settings-subsection-title", "Configured providers" }
                                    p { class: "settings-subsection-copy", "Saved keys are masked and cannot be revealed from this screen." }
                                }
                                div {
                                    id: "{test_ids::SETTINGS_LIST}",
                                    "data-testid": "{test_ids::SETTINGS_LIST}",
                                    class: "provider-list",
                                    if llm_settings.read().is_empty() {
                                        div { class: "empty-state", "No provider keys configured yet." }
                                    } else {
                                        for item in llm_settings.read().iter() {
                                            div { class: "provider-row", key: "{item.provider}",
                                                div { class: "provider-identity",
                                                    span { class: "provider-name", "{item.provider}" }
                                                    if let Some(url) = item.base_url.as_ref().and_then(|value| value.as_ref()) {
                                                        span { class: "provider-endpoint", title: "{url}", "{url}" }
                                                    }
                                                    span { class: "provider-meta",
                                                        if item.has_key { "API key saved · masked" } else { "No API key saved" }
                                                    }
                                                }
                                                div { class: "provider-badges",
                                                    if item.is_inherited {
                                                        span { class: "badge badge-neutral", "Inherited" }
                                                    } else if item.is_shared {
                                                        span { class: "badge badge-neutral", "Shared" }
                                                    } else if item.has_key {
                                                        span { class: "badge badge-success", "Configured" }
                                                    } else {
                                                        span { class: "badge badge-neutral", "No key" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        section {
                            id: "settings-usage",
                            class: "settings-card",
                            "aria-labelledby": "settings-usage-title",

                            div { class: "settings-card-head",
                                div {
                                    h2 { id: "settings-usage-title", class: "settings-title", "Token usage" }
                                    p { class: "settings-copy", "Per-request totals from recorded provider responses for this account." }
                                }
                                button {
                                    class: "secondary button-compact",
                                    r#type: "button",
                                    disabled: *token_loading.read() || *token_loading_older.read(),
                                    onclick: move |_| load_usage(),
                                    if *token_loading.read() || *token_loading_older.read() { "Refreshing…" } else { "Refresh" }
                                }
                            }

                            if token_usage.read().is_none() && token_error.read().is_none() {
                                div { class: "usage-loading", role: "status", "aria-label": "Loading token usage",
                                    div { class: "usage-metrics usage-metrics-skeleton", "aria-hidden": "true",
                                        div { class: "usage-metric", div { class: "skeleton skeleton-metric" } }
                                        div { class: "usage-metric", div { class: "skeleton skeleton-metric" } }
                                        div { class: "usage-metric", div { class: "skeleton skeleton-metric" } }
                                        div { class: "usage-metric", div { class: "skeleton skeleton-metric" } }
                                    }
                                    div { class: "skeleton skeleton-row", "aria-hidden": "true" }
                                    div { class: "skeleton skeleton-row", "aria-hidden": "true" }
                                    div { class: "skeleton skeleton-row", "aria-hidden": "true" }
                                    span { class: "visually-hidden", "Loading token usage…" }
                                }
                            } else if let Some(err) = token_error.read().as_ref() {
                                div { class: "error-banner", role: "alert", "{err}" }
                            } else if let Some(usage) = token_usage.read().as_ref() {
                                dl { class: "usage-metrics",
                                    div { class: "usage-metric",
                                        dt { class: "usage-metric-label", "Requests" }
                                        dd { class: "usage-metric-value", "{format_token_count(usage.totals.request_count)}" }
                                    }
                                    div { class: "usage-metric",
                                        dt { class: "usage-metric-label", "Input tokens" }
                                        dd { class: "usage-metric-value", "{format_token_count(usage.totals.input_tokens)}" }
                                    }
                                    div { class: "usage-metric",
                                        dt { class: "usage-metric-label", "Output tokens" }
                                        dd { class: "usage-metric-value", "{format_token_count(usage.totals.output_tokens)}" }
                                    }
                                    div { class: "usage-metric",
                                        dt { class: "usage-metric-label", "All tokens" }
                                        dd { class: "usage-metric-value usage-metric-value-accent", "{format_token_count(usage.totals.total_tokens)}" }
                                    }
                                }

                                if *token_loading.read() {
                                    div { class: "usage-refreshing", role: "status",
                                        div { class: "skeleton skeleton-row", "aria-hidden": "true" }
                                        div { class: "skeleton skeleton-row", "aria-hidden": "true" }
                                        span { class: "visually-hidden", "Refreshing token usage…" }
                                    }
                                } else if token_rows.read().is_empty() {
                                    div { class: "usage-empty", "No model request usage has been recorded yet." }
                                } else {
                                    div { class: "usage-table-scroll",
                                        table { class: "usage-table", "aria-label": "Token usage history",
                                            thead { tr {
                                                th { scope: "col", "Time" }
                                                th { scope: "col", "Model" }
                                                th { scope: "col", "Agent" }
                                                th { scope: "col", class: "usage-number", "Input" }
                                                th { scope: "col", class: "usage-number", "Output" }
                                                th { scope: "col", class: "usage-number", "Total" }
                                            } }
                                            tbody {
                                                for row in token_rows.read().iter() {
                                                    tr { key: "{row.id}",
                                                        td { class: "usage-time", "{format_usage_time(&row.timestamp)}" }
                                                        td {
                                                            span { class: "usage-model", "{row.model_name}" }
                                                            span { class: "usage-provider", "{row.model_provider}" }
                                                        }
                                                        td { class: "usage-agent", title: "{row.agent_id}", "{row.agent_id}" }
                                                        td { class: "usage-number", "{format_token_count(row.input_tokens)}" }
                                                        td { class: "usage-number", "{format_token_count(row.output_tokens)}" }
                                                        td { class: "usage-number usage-number-total", "{format_token_count(row.total_tokens)}" }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    if *token_older_error.read() {
                                        p { class: "inline-error", role: "alert", "Older requests could not be loaded." }
                                    }

                                    div { class: "usage-footer",
                                        p { class: "usage-footer-note", "{token_rows.read().len()} requests loaded; totals cover all recorded requests." }
                                        if token_cursor.read().is_some() {
                                            button {
                                                class: "secondary button-compact",
                                                r#type: "button",
                                                disabled: *token_loading_older.read() || *token_loading.read(),
                                                onclick: load_older_usage.clone(),
                                                if *token_loading_older.read() { "Loading…" } else { "Load older requests" }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        section {
                            id: "settings-server",
                            class: "settings-card",
                            "aria-labelledby": "settings-server-title",

                            div { class: "settings-card-head",
                                div {
                                    h2 { id: "settings-server-title", class: "settings-title", "Server connection" }
                                    p { class: "settings-copy", "Changing the server clears the active session and saved credentials." }
                                }
                            }

                            if let Some(err) = server_error.read().as_ref() {
                                div {
                                    class: "error-banner",
                                    role: "alert",
                                    id: "{test_ids::SETTINGS_SERVER_URL_ERROR_MESSAGE}",
                                    "data-testid": "{test_ids::SETTINGS_SERVER_URL_ERROR_MESSAGE}",
                                    "{err}"
                                }
                            }

                            label { class: "field-label", r#for: "{test_ids::SETTINGS_INPUT_SERVER_URL}", "API base URL" }
                            input {
                                id: "{test_ids::SETTINGS_INPUT_SERVER_URL}",
                                "data-testid": "{test_ids::SETTINGS_INPUT_SERVER_URL}",
                                "aria-describedby": "server-url-help",
                                placeholder: "https://api.example.com",
                                value: "{server_url_input}",
                                oninput: move |evt| server_url_input.set(evt.value().clone()),
                            }
                            p { id: "server-url-help", class: "field-help", "Use the base URL for your Mensetsu API." }

                            div { class: "settings-actions",
                                button {
                                    class: "primary",
                                    id: "{test_ids::SETTINGS_BUTTON_SAVE_SERVER_URL}",
                                    "data-testid": "{test_ids::SETTINGS_BUTTON_SAVE_SERVER_URL}",
                                    disabled: *is_saving_server.read(),
                                    onclick: handle_change_server,
                                    if *is_saving_server.read() { "Updating…" } else { "Change server" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
