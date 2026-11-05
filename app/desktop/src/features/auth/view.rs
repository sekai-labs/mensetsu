use dioxus::prelude::*;
use crate::services::api::{ApiError, ApiService};
use crate::ui::test_ids::test_ids;

#[derive(Props, Clone, PartialEq)]
pub struct AuthProps {
    pub api: ApiService,
    pub on_success: EventHandler<()>,
    pub on_change_server: EventHandler<()>,
}

#[component]
pub fn AuthView(props: AuthProps) -> Element {
    let mut is_register_mode = use_signal(|| false);
    let mut name_text = use_signal(String::new);
    let mut email_text = use_signal(String::new);
    let mut password_text = use_signal(String::new);
    let mut two_fa_code = use_signal(String::new);
    let mut temp_2fa_token = use_signal(|| Option::<String>::None);
    let mut error_msg = use_signal(|| Option::<String>::None);
    let mut is_submitting = use_signal(|| false);
    let api = props.api.clone();
    let on_success = props.on_success;

    let handle_submit = {
        let api = api.clone();
        move |_| {
            error_msg.set(None);
            is_submitting.set(true);
            let api = api.clone();
            let is_reg = *is_register_mode.read();
            let email = email_text.read().clone();
            let password = password_text.read().clone();
            let name = name_text.read().clone();

            spawn(async move {
                let res = if is_reg {
                    api.register(&name, &email, &password).await
                } else {
                    api.login(&email, &password).await
                };

                is_submitting.set(false);
                match res {
                    Ok(_) => {
                        on_success.call(());
                    }
                    Err(ApiError::Requires2Fa { temp_token }) => {
                        temp_2fa_token.set(Some(temp_token));
                    }
                    Err(e) => {
                        error_msg.set(Some(e.to_string()));
                    }
                }
            });
        }
    };

    let handle_2fa_submit = {
        let api = api.clone();
        move |_| {
            if let Some(temp) = temp_2fa_token.read().clone() {
                error_msg.set(None);
                is_submitting.set(true);
                let api = api.clone();
                let code = two_fa_code.read().clone();

                spawn(async move {
                    let res = api.login_2fa(&temp, &code).await;
                    is_submitting.set(false);
                    match res {
                        Ok(_) => {
                            on_success.call(());
                        }
                        Err(e) => {
                            error_msg.set(Some(e.to_string()));
                        }
                    }
                });
            }
        }
    };

    let has_2fa = temp_2fa_token.read().is_some();

    rsx! {
        div {
            class: "center-container",
            id: "{test_ids::AUTH_VIEW}",
            "data-testid": "{test_ids::AUTH_VIEW}",
            div {
                class: "card",
                div { style: "display: flex; justify-content: space-between; align-items: baseline; margin-bottom: 8px;",
                    h1 { class: "title-1", "Welcome to Mensetsu" }
                    button {
                        class: "secondary",
                        style: "font-size: 12px; padding: 4px 8px;",
                        onclick: move |_| props.on_change_server.call(()),
                        "Server"
                    }
                }
                p { class: "dim-label", style: "margin-bottom: 20px;", "Evidence-backed interview preparation" }

                if let Some(err) = error_msg.read().as_ref() {
                    div {
                        class: "error-banner",
                        id: "{test_ids::AUTH_ERROR_LABEL}",
                        "data-testid": "{test_ids::AUTH_ERROR_LABEL}",
                        "{err}"
                    }
                }

                if !has_2fa {
                    div { style: "display: flex; justify-content: flex-end; margin-bottom: 12px;",
                        button {
                            class: "secondary",
                            id: "{test_ids::AUTH_MODE_SWITCH}",
                            "data-testid": "{test_ids::AUTH_MODE_SWITCH}",
                            style: "font-size: 13px;",
                            onclick: move |_| {
                                let cur = *is_register_mode.read();
                                is_register_mode.set(!cur);
                                error_msg.set(None);
                            },
                            if *is_register_mode.read() {
                                "Already have an account? Sign In"
                            } else {
                                "Need an account? Register"
                            }
                        }
                    }

                    if *is_register_mode.read() {
                        div { style: "margin-bottom: 12px;",
                            label { class: "dim-label", style: "display: block; margin-bottom: 4px;", "Full Name" }
                            input {
                                id: "{test_ids::AUTH_NAME_INPUT}",
                                "data-testid": "{test_ids::AUTH_NAME_INPUT}",
                                placeholder: "Full Name",
                                value: "{name_text}",
                                oninput: move |evt| name_text.set(evt.value().clone()),
                            }
                        }
                    }

                    div { style: "margin-bottom: 12px;",
                        label { class: "dim-label", style: "display: block; margin-bottom: 4px;", "Email Address" }
                        input {
                            id: "{test_ids::AUTH_EMAIL_INPUT}",
                            "data-testid": "{test_ids::AUTH_EMAIL_INPUT}",
                            r#type: "email",
                            placeholder: "Email address",
                            value: "{email_text}",
                            oninput: move |evt| email_text.set(evt.value().clone()),
                        }
                    }

                    div { style: "margin-bottom: 20px;",
                        label { class: "dim-label", style: "display: block; margin-bottom: 4px;", "Password" }
                        input {
                            id: "{test_ids::AUTH_PASSWORD_INPUT}",
                            "data-testid": "{test_ids::AUTH_PASSWORD_INPUT}",
                            r#type: "password",
                            placeholder: "Password",
                            value: "{password_text}",
                            oninput: move |evt| password_text.set(evt.value().clone()),
                        }
                    }

                    button {
                        class: "primary",
                        id: "{test_ids::AUTH_SUBMIT_BUTTON}",
                        "data-testid": "{test_ids::AUTH_SUBMIT_BUTTON}",
                        style: "width: 100%;",
                        disabled: *is_submitting.read(),
                        onclick: handle_submit,
                        if *is_submitting.read() {
                            "Please wait..."
                        } else if *is_register_mode.read() {
                            "Create Account"
                        } else {
                            "Log In"
                        }
                    }
                } else {
                    // 2FA challenge mode
                    div { style: "margin-bottom: 16px;",
                        label { class: "dim-label", style: "display: block; margin-bottom: 6px;", "Two-Factor Authentication Code" }
                        input {
                            id: "{test_ids::AUTH_2FA_INPUT}",
                            "data-testid": "{test_ids::AUTH_2FA_INPUT}",
                            placeholder: "6-digit 2FA code",
                            value: "{two_fa_code}",
                            oninput: move |evt| two_fa_code.set(evt.value().clone()),
                        }
                    }

                    div { style: "display: flex; gap: 12px;",
                        button {
                            class: "secondary",
                            style: "flex: 1;",
                            onclick: move |_| {
                                temp_2fa_token.set(None);
                                two_fa_code.set(String::new());
                                error_msg.set(None);
                            },
                            "Cancel"
                        }
                        button {
                            class: "primary",
                            id: "{test_ids::AUTH_2FA_SUBMIT}",
                            "data-testid": "{test_ids::AUTH_2FA_SUBMIT}",
                            style: "flex: 2;",
                            disabled: *is_submitting.read(),
                            onclick: handle_2fa_submit,
                            if *is_submitting.read() { "Verifying..." } else { "Submit Code" }
                        }
                    }
                }
            }
        }
    }
}
