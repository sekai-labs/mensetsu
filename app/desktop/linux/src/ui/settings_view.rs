use gtk4::prelude::*;
use gtk4::{
    Box, Button, Entry, Label, ListBox, ListBoxRow, Orientation, PasswordEntry, ScrolledWindow,
};
use std::rc::Rc;
use mensetsu_contracts::models::UpsertLlmConfigRequestDto;
use crate::api_service::ApiService;
use crate::test_ids::ids;

pub struct SettingsView {
    pub container: Box,
}

impl SettingsView {
    pub fn new<FBack, FServerChanged>(
        api: ApiService,
        on_back: FBack,
        on_server_changed: FServerChanged,
    ) -> Self
    where
        FBack: Fn() + 'static,
        FServerChanged: Fn() + 'static,
    {
        let container = Box::new(Orientation::Vertical, 12);
        container.set_widget_name(ids::SETTINGS_VIEW);
        container.set_margin_start(24);
        container.set_margin_end(24);
        container.set_margin_top(16);
        container.set_margin_bottom(16);

        // Header with Back button
        let header = Box::new(Orientation::Horizontal, 12);
        let back_btn = Button::with_label("← Back");
        back_btn.set_widget_name(ids::SETTINGS_BACK_BUTTON);

        let title = Label::new(Some("Settings"));
        title.add_css_class("title-2");
        title.set_hexpand(true);
        title.set_halign(gtk4::Align::Start);

        header.append(&back_btn);
        header.append(&title);

        back_btn.connect_clicked(move |_| {
            on_back();
        });

        // Server Endpoint Section
        let server_box = Box::new(Orientation::Vertical, 8);
        let server_title = Label::new(Some("Rust API Server Endpoint"));
        server_title.set_halign(gtk4::Align::Start);
        server_title.add_css_class("heading");

        let server_desc = Label::new(Some(
            "Changing the server resets your active session and credentials so tokens do not leak.",
        ));
        server_desc.set_wrap(true);
        server_desc.set_halign(gtk4::Align::Start);
        server_desc.add_css_class("dim-label");

        let server_entry = Entry::new();
        server_entry.set_widget_name(ids::SETTINGS_INPUT_SERVER_URL);
        let current_base = api.get_base_url();
        server_entry.set_text(&current_base);
        server_entry.set_placeholder_text(Some("https://api.example.com"));

        let server_error_label = Label::new(None);
        server_error_label.set_widget_name(ids::SETTINGS_SERVER_URL_ERROR_MESSAGE);
        server_error_label.add_css_class("error");
        server_error_label.set_visible(false);

        let save_server_btn = Button::with_label("Change Server");
        save_server_btn.set_widget_name(ids::SETTINGS_BUTTON_SAVE_SERVER_URL);
        save_server_btn.add_css_class("suggested-action");

        server_box.append(&server_title);
        server_box.append(&server_desc);
        server_box.append(&server_entry);
        server_box.append(&server_error_label);
        server_box.append(&save_server_btn);

        // Server change action
        {
            let api = api.clone();
            let server_entry = server_entry.clone();
            let server_error = server_error_label.clone();
            let on_server_changed = Rc::new(on_server_changed);

            save_server_btn.connect_clicked(move |_| {
                let raw = server_entry.text().to_string();
                match crate::endpoint_config::validate_and_normalize_url(&raw) {
                    Ok(normalized) => {
                        server_error.set_visible(false);
                        let api = api.clone();
                        let on_server_changed = on_server_changed.clone();
                        let server_error = server_error.clone();

                        glib::MainContext::default().spawn_local(async move {
                            // 1. Reset in-memory tokens, keyring refresh token, and set new URL
                            if let Err(e) = api.set_base_url(normalized.clone()).await {
                                server_error.set_text(&format!("Failed to reset session for new server: {e}"));
                                server_error.set_visible(true);
                                return;
                            }

                            // 2. Persist only after session reset succeeds
                            if let Err(e) = crate::endpoint_config::set_persisted_server_url(&normalized) {
                                server_error.set_text(&format!("Failed to save configuration: {e}"));
                                server_error.set_visible(true);
                                return;
                            }

                            // 3. Navigate away
                            on_server_changed();
                        });
                    }
                    Err(e) => {
                        server_error.set_text(&e.to_string());
                        server_error.set_visible(true);
                    }
                }
            });
        }

        // Form to configure provider key
        let form_box = Box::new(Orientation::Vertical, 8);
        let form_title = Label::new(Some("Add / Update Provider Key"));
        form_title.set_halign(gtk4::Align::Start);
        form_title.add_css_class("heading");

        let provider_entry = Entry::new();
        provider_entry.set_widget_name(ids::SETTINGS_PROVIDER_INPUT);
        provider_entry.set_placeholder_text(Some("Provider (e.g. openai, anthropic)"));

        let key_entry = PasswordEntry::new();
        key_entry.set_widget_name(ids::SETTINGS_KEY_INPUT);
        key_entry.set_show_peek_icon(true);

        let save_btn = Button::with_label("Save Setting");
        save_btn.set_widget_name(ids::SETTINGS_SAVE_BUTTON);
        save_btn.add_css_class("suggested-action");

        form_box.append(&form_title);
        form_box.append(&provider_entry);
        form_box.append(&key_entry);
        form_box.append(&save_btn);
        // Current settings list
        let current_title = Label::new(Some("Configured Providers:"));
        current_title.set_halign(gtk4::Align::Start);
        current_title.add_css_class("heading");

        let scrolled = ScrolledWindow::new();
        scrolled.set_vexpand(true);
        let settings_list = ListBox::new();
        settings_list.set_widget_name(ids::SETTINGS_LIST);
        settings_list.add_css_class("boxed-list");
        scrolled.set_child(Some(&settings_list));

        let status_label = Label::new(None);
        status_label.set_visible(false);

        container.append(&header);
        container.append(&server_box);
        container.append(&form_box);
        container.append(&status_label);
        container.append(&current_title);
        container.append(&scrolled);

        let reload_settings = {
            let api = api.clone();
            let settings_list = settings_list.clone();
            let status_label = status_label.clone();

            Rc::new(move || {
                let api = api.clone();
                let settings_list = settings_list.clone();
                let status_label = status_label.clone();

                glib::MainContext::default().spawn_local(async move {
                    while let Some(c) = settings_list.first_child() {
                        settings_list.remove(&c);
                    }
                    match api.get_llm_settings().await {
                        Ok(items) => {
                            for item in items {
                                let row = ListBoxRow::new();
                                let b = Box::new(Orientation::Horizontal, 12);
                                b.set_margin_start(12);
                                b.set_margin_end(12);
                                b.set_margin_top(8);
                                b.set_margin_bottom(8);

                                let p_label = Label::new(Some(&item.provider));
                                p_label.set_hexpand(true);
                                p_label.set_halign(gtk4::Align::Start);

                                let key_hint = if item.has_key { "Key configured" } else { "No key" };
                                let h_label = Label::new(Some(key_hint));
                                h_label.add_css_class("dim-label");

                                b.append(&p_label);
                                b.append(&h_label);
                                row.set_child(Some(&b));
                                settings_list.append(&row);
                            }
                        }
                        Err(e) => {
                            status_label.set_text(&format!("Error: {e}"));
                            status_label.set_visible(true);
                        }
                    }
                });
            })
        };

        // Save action
        {
            let api = api.clone();
            let provider_entry = provider_entry.clone();
            let key_entry = key_entry.clone();
            let status_label = status_label.clone();
            let reload = reload_settings.clone();

            save_btn.connect_clicked(move |_| {
                let provider = provider_entry.text().to_string();
                let key = key_entry.text().to_string();

                if provider.is_empty() || key.is_empty() {
                    status_label.set_text("Provider and Key are required.");
                    status_label.set_visible(true);
                    return;
                }

                status_label.set_visible(false);
                let api = api.clone();
                let status_label = status_label.clone();
                let reload = reload.clone();
                let provider_entry = provider_entry.clone();
                let key_entry = key_entry.clone();

                glib::MainContext::default().spawn_local(async move {
                    let mut req = UpsertLlmConfigRequestDto::default();
                    req.provider = provider;
                    req.api_key = Some(Some(key));

                    match api.update_llm_setting(req).await {
                        Ok(_) => {
                            provider_entry.set_text("");
                            key_entry.set_text("");
                            reload();
                        }
                        Err(e) => {
                            status_label.set_text(&format!("Failed to save setting: {e}"));
                            status_label.set_visible(true);
                        }
                    }
                });
            });
        }

        reload_settings();

        Self { container }
    }
}
