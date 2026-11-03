use gtk4::prelude::*;
use gtk4::{Box, Button, Entry, Label, Orientation};
use std::rc::Rc;
use crate::api_service::ApiService;
use crate::endpoint_config::{set_persisted_server_url, validate_and_normalize_url};
use crate::test_ids::ids;

pub struct ServerSetupView {
    pub container: Box,
}

impl ServerSetupView {
    pub fn new<FOnSaved, FCancel>(
        api: ApiService,
        current_url: Option<String>,
        on_saved: FOnSaved,
        on_cancel: Option<FCancel>,
    ) -> Self
    where
        FOnSaved: Fn(String) + 'static,
        FCancel: Fn() + 'static,
    {
        let container = Box::new(Orientation::Vertical, 16);
        container.set_widget_name(ids::SERVER_SETUP_SCREEN);
        container.set_margin_start(40);
        container.set_margin_end(40);
        container.set_margin_top(60);
        container.set_margin_bottom(60);
        container.set_halign(gtk4::Align::Center);
        container.set_valign(gtk4::Align::Center);

        let title_label = Label::new(Some("Connect to Mensetsu Server"));
        title_label.add_css_class("title-1");

        let desc_label = Label::new(Some(
            "Enter your self-hosted Rust API base URL (e.g. https://api.example.com or http://127.0.0.1:3000 for local development).",
        ));
        desc_label.set_wrap(true);
        desc_label.set_max_width_chars(50);
        desc_label.add_css_class("dim-label");

        let url_entry = Entry::new();
        url_entry.set_widget_name(ids::INPUT_SERVER_URL);
        url_entry.set_placeholder_text(Some("https://api.example.com"));
        url_entry.set_width_chars(40);
        if let Some(curr) = current_url {
            url_entry.set_text(&curr);
        }

        let error_label = Label::new(None);
        error_label.set_widget_name(ids::SERVER_URL_ERROR_MESSAGE);
        error_label.add_css_class("error");
        error_label.set_wrap(true);
        error_label.set_max_width_chars(50);
        error_label.set_visible(false);

        let button_box = Box::new(Orientation::Horizontal, 12);
        button_box.set_halign(gtk4::Align::Center);

        let save_button = Button::with_label("Connect");
        save_button.set_widget_name(ids::BUTTON_SAVE_SERVER_URL);
        save_button.add_css_class("suggested-action");
        button_box.append(&save_button);

        if let Some(cancel_fn) = on_cancel {
            let cancel_btn = Button::with_label("Cancel");
            cancel_btn.connect_clicked(move |_| {
                cancel_fn();
            });
            button_box.append(&cancel_btn);
        }

        let on_saved_rc = Rc::new(on_saved);

        {
            let url_entry = url_entry.clone();
            let error_label = error_label.clone();
            let on_saved = on_saved_rc.clone();
            let api = api.clone();

            save_button.connect_clicked(move |_| {
                let raw_input = url_entry.text().to_string();
                match validate_and_normalize_url(&raw_input) {
                    Ok(normalized) => {
                        error_label.set_visible(false);
                        let api = api.clone();
                        let error_label = error_label.clone();
                        let on_saved = on_saved.clone();

                        glib::MainContext::default().spawn_local(async move {
                            // 1. Reset tokens / credentials and set active base URL in memory
                            if let Err(e) = api.set_base_url(normalized.clone()).await {
                                error_label.set_text(&format!("Failed to reset session: {e}"));
                                error_label.set_visible(true);
                                return;
                            }

                            // 2. Persist to disk only after session reset succeeds
                            if let Err(e) = set_persisted_server_url(&normalized) {
                                error_label.set_text(&format!("Failed to save configuration: {e}"));
                                error_label.set_visible(true);
                                return;
                            }

                            // 3. Callback to proceed to auth/next view
                            on_saved(normalized);
                        });
                    }
                    Err(e) => {
                        error_label.set_text(&e.to_string());
                        error_label.set_visible(true);
                    }
                }
            });
        }
        container.append(&title_label);
        container.append(&desc_label);
        container.append(&url_entry);
        container.append(&error_label);
        container.append(&button_box);

        Self { container }
    }
}
