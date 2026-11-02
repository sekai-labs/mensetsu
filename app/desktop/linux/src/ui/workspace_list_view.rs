use gtk4::prelude::*;
use gtk4::{
    Box, Button, Dialog, Entry, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow,
    TextView, Window,
};
use std::{cell::RefCell, path::PathBuf, rc::Rc};
use mensetsu_contracts::models::WorkspaceResponseDto;
use crate::api_service::ApiService;
use crate::test_ids::ids;

pub struct WorkspaceListView {
    pub container: Box,
}

impl WorkspaceListView {
    pub fn new<FSelect, FSettings, FLogout>(
        api: ApiService,
        on_select_workspace: FSelect,
        on_settings: FSettings,
        on_logout: FLogout,
    ) -> Self
    where
        FSelect: Fn(String, String) + 'static, // (id, title)
        FSettings: Fn() + 'static,
        FLogout: Fn() + 'static,
    {
        let container = Box::new(Orientation::Vertical, 12);
        container.set_widget_name(ids::MAIN_VIEW);
        container.set_margin_start(24);
        container.set_margin_end(24);
        container.set_margin_top(16);
        container.set_margin_bottom(16);

        // Header
        let header = Box::new(Orientation::Horizontal, 12);
        let title = Label::new(Some("Workspaces"));
        title.add_css_class("title-2");
        title.set_hexpand(true);
        title.set_halign(gtk4::Align::Start);

        let new_btn = Button::with_label("+ New Workspace");
        new_btn.set_widget_name(ids::NEW_WORKSPACE_BUTTON);
        new_btn.add_css_class("suggested-action");

        let refresh_btn = Button::with_label("Refresh");
        refresh_btn.set_widget_name(ids::REFRESH_WORKSPACES_BUTTON);

        let settings_btn = Button::with_label("Settings");
        settings_btn.set_widget_name(ids::SETTINGS_BUTTON);

        let logout_btn = Button::with_label("Log Out");
        logout_btn.set_widget_name(ids::LOGOUT_BUTTON);
        logout_btn.add_css_class("destructive-action");

        header.append(&title);
        header.append(&new_btn);
        header.append(&refresh_btn);
        header.append(&settings_btn);
        header.append(&logout_btn);

        // Filters: All / Ready / Processing
        let filter_box = Box::new(Orientation::Horizontal, 8);
        let filter_all = Button::with_label("All");
        filter_all.set_widget_name(ids::FILTER_ALL_BUTTON);
        let filter_ready = Button::with_label("Ready");
        filter_ready.set_widget_name(ids::FILTER_READY_BUTTON);
        let filter_processing = Button::with_label("Processing");
        filter_processing.set_widget_name(ids::FILTER_PROCESSING_BUTTON);

        filter_box.append(&filter_all);
        filter_box.append(&filter_ready);
        filter_box.append(&filter_processing);

        // Status / Error message label
        let status_label = Label::new(None);
        status_label.set_visible(false);

        // List container
        let scrolled = ScrolledWindow::new();
        scrolled.set_vexpand(true);
        let list_box = ListBox::new();
        list_box.set_widget_name(ids::WORKSPACE_LIST);
        list_box.add_css_class("boxed-list");
        scrolled.set_child(Some(&list_box));

        container.append(&header);
        container.append(&filter_box);
        container.append(&status_label);
        container.append(&scrolled);

        let on_select_rc = Rc::new(on_select_workspace);
        let on_logout_rc = Rc::new(on_logout);
        let workspaces_state: Rc<std::cell::RefCell<Vec<WorkspaceResponseDto>>> =
            Rc::new(std::cell::RefCell::new(Vec::new()));
        let current_filter = Rc::new(std::cell::RefCell::new("all".to_string()));

        let populate_list = {
            let list_box = list_box.clone();
            let workspaces_state = workspaces_state.clone();
            let current_filter = current_filter.clone();
            let on_select = on_select_rc.clone();

            Rc::new(move || {
                // Clear existing
                while let Some(child) = list_box.first_child() {
                    list_box.remove(&child);
                }

                let items = workspaces_state.borrow();
                let filter = current_filter.borrow();
                let mut count = 0;
                for ws in items.iter() {
                    let status_str = format!("{}", ws.status);
                    if filter.as_str() == "ready" && !status_str.eq_ignore_ascii_case("ready") && !status_str.eq_ignore_ascii_case("completed") {
                        continue;
                    }
                    if filter.as_str() == "processing" && !status_str.eq_ignore_ascii_case("processing") && !status_str.eq_ignore_ascii_case("running") {
                        continue;
                    }

                    count += 1;
                    let row = ListBoxRow::new();
                    let item_id = ws.id.clone();
                    row.set_widget_name(&format!("{}{}", ids::WORKSPACE_ITEM_PREFIX, item_id));

                    let row_box = Box::new(Orientation::Horizontal, 12);
                    row_box.set_margin_start(12);
                    row_box.set_margin_end(12);
                    row_box.set_margin_top(8);
                    row_box.set_margin_bottom(8);

                    let title_text = format!("{} — {}", ws.company_name, ws.role_title);
                    let label = Label::new(Some(&title_text));
                    label.set_hexpand(true);
                    label.set_halign(gtk4::Align::Start);

                    let status_badge = Label::new(Some(&status_str));
                    status_badge.add_css_class("dim-label");

                    row_box.append(&label);
                    row_box.append(&status_badge);
                    row.set_child(Some(&row_box));

                    let on_select = on_select.clone();
                    let target_id = ws.id.clone();
                    let title_for_click = title_text.clone();
                    row.connect_activate(move |_| {
                        on_select(target_id.clone(), title_for_click.clone());
                    });

                    list_box.append(&row);
                }

                if count == 0 {
                    let empty_label = Label::new(Some("No workspaces found."));
                    empty_label.set_margin_top(20);
                    empty_label.set_margin_bottom(20);
                    empty_label.add_css_class("dim-label");
                    list_box.append(&empty_label);
                }
            })
        };

        // Reload data from API
        let reload_workspaces = {
            let api = api.clone();
            let status_label = status_label.clone();
            let workspaces_state = workspaces_state.clone();
            let populate = populate_list.clone();

            Rc::new(move || {
                let api = api.clone();
                let status_label = status_label.clone();
                let workspaces_state = workspaces_state.clone();
                let populate = populate.clone();

                status_label.set_text("Loading workspaces...");
                status_label.set_visible(true);

                glib::MainContext::default().spawn_local(async move {
                    match api.list_workspaces().await {
                        Ok(list) => {
                            *workspaces_state.borrow_mut() = list;
                            status_label.set_visible(false);
                            populate();
                        }
                        Err(e) => {
                            status_label.set_text(&format!("Error loading workspaces: {e}"));
                            status_label.set_visible(true);
                        }
                    }
                });
            })
        };

        // Connect filters
        {
            let current_filter = current_filter.clone();
            let populate = populate_list.clone();
            filter_all.connect_clicked(move |_| {
                *current_filter.borrow_mut() = "all".to_string();
                populate();
            });
        }
        {
            let current_filter = current_filter.clone();
            let populate = populate_list.clone();
            filter_ready.connect_clicked(move |_| {
                *current_filter.borrow_mut() = "ready".to_string();
                populate();
            });
        }
        {
            let current_filter = current_filter.clone();
            let populate = populate_list.clone();
            filter_processing.connect_clicked(move |_| {
                *current_filter.borrow_mut() = "processing".to_string();
                populate();
            });
        }

        // Refresh button
        {
            let reload = reload_workspaces.clone();
            refresh_btn.connect_clicked(move |_| {
                reload();
            });
        }

        // Settings button
        settings_btn.connect_clicked(move |_| {
            on_settings();
        });

        // Logout button
        {
            let api = api.clone();
            let on_logout = on_logout_rc.clone();
            let status_label = status_label.clone();
            logout_btn.connect_clicked(move |_| {
                let api = api.clone();
                let on_logout = on_logout.clone();
                let status_label = status_label.clone();
                glib::MainContext::default().spawn_local(async move {
                    match api.logout().await {
                        Ok(()) => on_logout(),
                        Err(e) => {
                            status_label.set_text(&format!("Logout failed: {e}"));
                            status_label.set_visible(true);
                        }
                    }
                });
            });
        }

        // New Workspace Dialog
        {
            let api = api.clone();
            let reload = reload_workspaces.clone();
            let container_widget = container.clone();

            new_btn.connect_clicked(move |_| {
                let root_win = container_widget.root().and_downcast::<Window>();
                let dialog = Dialog::with_buttons(
                    Some("Create Preparation Workspace"),
                    root_win.as_ref(),
                    gtk4::DialogFlags::MODAL,
                    &[
                        ("Cancel", gtk4::ResponseType::Cancel),
                        ("Create", gtk4::ResponseType::Accept),
                    ],
                );
                dialog.set_default_size(500, 450);

                let content_area = dialog.content_area();
                content_area.set_spacing(10);
                content_area.set_margin_start(16);
                content_area.set_margin_end(16);
                content_area.set_margin_top(16);
                content_area.set_margin_bottom(16);

                let company_entry = Entry::new();
                company_entry.set_widget_name(ids::CREATE_COMPANY_INPUT);
                company_entry.set_placeholder_text(Some("Target Company (e.g. Acme Inc)"));

                let role_entry = Entry::new();
                role_entry.set_widget_name(ids::CREATE_ROLE_INPUT);
                role_entry.set_placeholder_text(Some("Role Title (e.g. Senior Rust Engineer)"));

                let resume_label = Label::new(Some("Résumé Text:"));
                resume_label.set_halign(gtk4::Align::Start);
                let resume_tv = TextView::new();
                resume_tv.set_widget_name(ids::CREATE_RESUME_INPUT);
                let resume_scroll = ScrolledWindow::new();
                resume_scroll.set_min_content_height(80);
                resume_scroll.set_child(Some(&resume_tv));
                let pdf_button = Button::with_label("Choose résumé PDF…");
                pdf_button.set_widget_name(ids::CREATE_PDF_BUTTON);
                let pdf_label = Label::new(Some("Or select a PDF (maximum 5 MiB)"));
                pdf_label.set_widget_name(ids::CREATE_PDF_LABEL);
                pdf_label.set_halign(gtk4::Align::Start);
                let selected_pdf: Rc<RefCell<Option<PathBuf>>> = Rc::new(RefCell::new(None));

                let jd_label = Label::new(Some("Job Description:"));
                jd_label.set_halign(gtk4::Align::Start);
                let jd_tv = TextView::new();
                jd_tv.set_widget_name(ids::CREATE_JD_INPUT);
                let jd_scroll = ScrolledWindow::new();
                jd_scroll.set_min_content_height(80);
                jd_scroll.set_child(Some(&jd_tv));

                let err_label = Label::new(None);
                err_label.set_widget_name(ids::CREATE_ERROR_LABEL);
                err_label.add_css_class("error");
                err_label.set_visible(false);

                content_area.append(&company_entry);
                content_area.append(&role_entry);
                content_area.append(&resume_label);
                content_area.append(&resume_scroll);
                content_area.append(&pdf_button);
                content_area.append(&pdf_label);
                content_area.append(&jd_label);
                content_area.append(&jd_scroll);
                content_area.append(&err_label);

                // Set accessible widget names on action buttons
                if let Some(cancel_btn) = dialog.widget_for_response(gtk4::ResponseType::Cancel) {
                    cancel_btn.set_widget_name(ids::CREATE_CANCEL_BUTTON);
                }
                let create_btn = dialog.widget_for_response(gtk4::ResponseType::Accept)
                    .and_downcast::<Button>()
                    .expect("create dialog has an Accept button");
                create_btn.set_widget_name(ids::CREATE_SUBMIT_BUTTON);

                let api = api.clone();
                let reload = reload.clone();
                {
                    let dialog = dialog.clone();
                    let selected_pdf = selected_pdf.clone();
                    let pdf_label = pdf_label.clone();
                    let err_label = err_label.clone();
                    pdf_button.connect_clicked(move |_| {
                        let chooser = gtk4::FileChooserDialog::new(
                            Some("Select résumé PDF"),
                            Some(&dialog),
                            gtk4::FileChooserAction::Open,
                            &[("Cancel", gtk4::ResponseType::Cancel), ("Choose", gtk4::ResponseType::Accept)],
                        );
                        let filter = gtk4::FileFilter::new();
                        filter.set_name(Some("PDF documents"));
                        filter.add_pattern("*.pdf");
                        filter.add_pattern("*.PDF");
                        chooser.add_filter(&filter);
                        let selected_pdf = selected_pdf.clone();
                        let pdf_label = pdf_label.clone();
                        let err_label = err_label.clone();
                        chooser.connect_response(move |chooser, response| {
                            if response == gtk4::ResponseType::Accept {
                                match chooser.file().and_then(|file| file.path()) {
                                    Some(path) => {
                                        pdf_label.set_text(&format!("Selected: {}", path.file_name()
                                            .and_then(|name| name.to_str()).unwrap_or("résumé PDF")));
                                        *selected_pdf.borrow_mut() = Some(path);
                                        err_label.set_visible(false);
                                    }
                                    None => {
                                        err_label.set_text("Choose a local PDF file.");
                                        err_label.set_visible(true);
                                    }
                                }
                            }
                            chooser.close();
                        });
                        chooser.present();
                    });
                }

                dialog.connect_response(move |d, resp| {
                    if resp == gtk4::ResponseType::Accept {
                        let company = company_entry.text().to_string();
                        let role = role_entry.text().to_string();

                        let res_buf = resume_tv.buffer();
                        let resume_text = res_buf.text(&res_buf.start_iter(), &res_buf.end_iter(), false).to_string();

                        let jd_buf = jd_tv.buffer();
                        let jd_text = jd_buf.text(&jd_buf.start_iter(), &jd_buf.end_iter(), false).to_string();
                        let pdf_path = selected_pdf.borrow().clone();
                        if pdf_path.is_some() && !resume_text.trim().is_empty() {
                            err_label.set_text("Choose either résumé text or a PDF, not both.");
                            err_label.set_visible(true);
                            return;
                        }

                        if company.is_empty() || role.is_empty() {
                            err_label.set_text("Company and Role are required.");
                            err_label.set_visible(true);
                            return;
                        }

                        let api = api.clone();
                        let create_btn = create_btn.clone();
                        create_btn.set_sensitive(false);
                        err_label.set_text("Creating workspace…");
                        err_label.set_visible(true);
                        let reload = reload.clone();
                        let d = d.clone();
                        let err_label = err_label.clone();

                        glib::MainContext::default().spawn_local(async move {
                            let result = if let Some(path) = pdf_path {
                                api.create_workspace_with_pdf(&company, &role, &jd_text, &path).await
                            } else {
                                api.create_workspace(&company, &role, &resume_text, &jd_text).await
                            };
                            match result {
                                Ok(_) => {
                                    d.close();
                                    reload();
                                }
                                Err(e) => {
                                    err_label.set_text(&format!("Create failed: {e}"));
                                    err_label.set_visible(true);
                                    create_btn.set_sensitive(true);
                                }
                            }
                        });
                    } else {
                        d.close();
                    }
                });

                dialog.present();
            });
        }

        // Initial fetch
        reload_workspaces();

        Self { container }
    }
}
