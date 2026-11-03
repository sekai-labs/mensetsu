use gtk4::prelude::*;
use gtk4::{Box, Button, Entry, Label, Orientation, PasswordEntry};
use std::rc::Rc;
use crate::api_service::ApiService;
use crate::test_ids::ids;

pub struct AuthView {
    pub container: Box,
}

impl AuthView {
    pub fn new<FSuccess, FChangeServer>(
        api: ApiService,
        on_success: FSuccess,
        on_change_server: FChangeServer,
    ) -> Self
    where
        FSuccess: Fn() + 'static,
        FChangeServer: Fn() + 'static,
    {
        let container = Box::new(Orientation::Vertical, 16);
        container.set_widget_name(ids::AUTH_VIEW);
        container.set_margin_start(40);
        container.set_margin_end(40);
        container.set_margin_top(60);
        container.set_margin_bottom(60);
        container.set_halign(gtk4::Align::Center);
        container.set_valign(gtk4::Align::Center);

        let title_label = Label::new(Some("Welcome to Mensetsu"));
        title_label.add_css_class("title-1");

        let subtitle_label = Label::new(Some("Evidence-backed interview preparation"));
        subtitle_label.add_css_class("dim-label");

        let mode_switch = Button::with_label("Need an account? Register");
        mode_switch.set_widget_name(ids::AUTH_MODE_SWITCH);
        mode_switch.add_css_class("flat");

        let name_entry = Entry::new();
        name_entry.set_widget_name(ids::AUTH_NAME_INPUT);
        name_entry.set_placeholder_text(Some("Full Name"));
        name_entry.set_visible(false); // Initially hidden (login mode)

        let email_entry = Entry::new();
        email_entry.set_widget_name(ids::AUTH_EMAIL_INPUT);
        email_entry.set_placeholder_text(Some("Email address"));

        let password_entry = PasswordEntry::new();
        password_entry.set_widget_name(ids::AUTH_PASSWORD_INPUT);
        password_entry.set_show_peek_icon(true);

        // 2FA code entry (initially hidden)
        let two_fa_entry = Entry::new();
        two_fa_entry.set_widget_name(ids::AUTH_2FA_INPUT);
        two_fa_entry.set_placeholder_text(Some("6-digit 2FA code"));
        two_fa_entry.set_visible(false);

        let submit_button = Button::with_label("Log In");
        submit_button.set_widget_name(ids::AUTH_SUBMIT_BUTTON);
        submit_button.add_css_class("suggested-action");

        let two_fa_submit = Button::with_label("Submit 2FA Code");
        two_fa_submit.set_widget_name(ids::AUTH_2FA_SUBMIT);
        two_fa_submit.add_css_class("suggested-action");
        two_fa_submit.set_visible(false);

        let error_label = Label::new(None);
        error_label.set_widget_name(ids::AUTH_ERROR_LABEL);
        error_label.add_css_class("error");
        error_label.set_wrap(true);
        error_label.set_visible(false);

        let is_register_mode = Rc::new(std::cell::Cell::new(false));
        let temp_2fa_token = Rc::new(std::cell::RefCell::new(Option::<String>::None));

        let is_reg_clone = is_register_mode.clone();
        let name_clone = name_entry.clone();
        let submit_clone = submit_button.clone();
        let mode_clone = mode_switch.clone();
        let err_clone = error_label.clone();
        mode_switch.connect_clicked(move |_| {
            let is_reg = !is_reg_clone.get();
            is_reg_clone.set(is_reg);
            name_clone.set_visible(is_reg);
            err_clone.set_visible(false);
            if is_reg {
                submit_clone.set_label("Register");
                mode_clone.set_label("Already have an account? Log In");
            } else {
                submit_clone.set_label("Log In");
                mode_clone.set_label("Need an account? Register");
            }
        });

        let on_success_rc = Rc::new(on_success);

        // Login / Register click handler
        {
            let api = api.clone();
            let is_reg_mode = is_register_mode.clone();
            let name_entry = name_entry.clone();
            let email_entry = email_entry.clone();
            let password_entry = password_entry.clone();
            let two_fa_entry = two_fa_entry.clone();
            let two_fa_submit = two_fa_submit.clone();
            let submit_btn = submit_button.clone();
            let error_label = error_label.clone();
            let temp_token_rc = temp_2fa_token.clone();
            let on_success = on_success_rc.clone();

            submit_button.connect_clicked(move |_| {
                let email = email_entry.text().to_string();
                let password = password_entry.text().to_string();
                let is_reg = is_reg_mode.get();
                let name = name_entry.text().to_string();

                if email.is_empty() || password.is_empty() {
                    error_label.set_text("Email and password are required.");
                    error_label.set_visible(true);
                    return;
                }
                if is_reg && name.is_empty() {
                    error_label.set_text("Name is required for registration.");
                    error_label.set_visible(true);
                    return;
                }

                error_label.set_visible(false);
                submit_btn.set_sensitive(false);

                let api = api.clone();
                let error_label = error_label.clone();
                let submit_btn = submit_btn.clone();
                let two_fa_entry = two_fa_entry.clone();
                let two_fa_submit = two_fa_submit.clone();
                let temp_token_rc = temp_token_rc.clone();
                let on_success = on_success.clone();

                glib::MainContext::default().spawn_local(async move {
                    let result = if is_reg {
                        api.register(&name, &email, &password).await
                    } else {
                        api.login(&email, &password).await
                    };

                    submit_btn.set_sensitive(true);

                    match result {
                        Ok(()) => {
                            on_success();
                        }
                        Err(crate::api_service::ApiError::Requires2Fa { temp_token }) => {
                            *temp_token_rc.borrow_mut() = Some(temp_token);
                            two_fa_entry.set_visible(true);
                            two_fa_submit.set_visible(true);
                            submit_btn.set_visible(false);
                            error_label.set_text("2FA code required");
                            error_label.set_visible(true);
                        }
                        Err(e) => {
                            error_label.set_text(&e.to_string());
                            error_label.set_visible(true);
                        }
                    }
                });
            });
        }

        // 2FA Submit click handler
        {
            let api = api.clone();
            let two_fa_entry = two_fa_entry.clone();
            let two_fa_submit_btn = two_fa_submit.clone();
            let error_label = error_label.clone();
            let temp_token_rc = temp_2fa_token.clone();
            let on_success = on_success_rc.clone();

            two_fa_submit.connect_clicked(move |_| {
                let code = two_fa_entry.text().to_string();
                let temp_token = temp_token_rc.borrow().clone().unwrap_or_default();

                if code.is_empty() {
                    error_label.set_text("Please enter your 2FA code.");
                    error_label.set_visible(true);
                    return;
                }

                error_label.set_visible(false);
                two_fa_submit_btn.set_sensitive(false);

                let api = api.clone();
                let error_label = error_label.clone();
                let two_fa_submit_btn = two_fa_submit_btn.clone();
                let on_success = on_success.clone();

                glib::MainContext::default().spawn_local(async move {
                    let res = api.login_2fa(&temp_token, &code).await;
                    two_fa_submit_btn.set_sensitive(true);

                    match res {
                        Ok(()) => {
                            on_success();
                        }
                        Err(e) => {
                            error_label.set_text(&e.to_string());
                            error_label.set_visible(true);
                        }
                    }
                });
            });
        }

        container.append(&title_label);
        container.append(&subtitle_label);
        container.append(&mode_switch);
        container.append(&name_entry);
        container.append(&email_entry);
        container.append(&password_entry);
        container.append(&two_fa_entry);
        container.append(&two_fa_submit);
        container.append(&submit_button);
        container.append(&error_label);

        let change_server_btn = Button::with_label("Change Server");
        change_server_btn.set_widget_name("auth_change_server_button");
        change_server_btn.add_css_class("flat");
        change_server_btn.connect_clicked(move |_| {
            on_change_server();
        });
        container.append(&change_server_btn);

        Self { container }
    }
}
