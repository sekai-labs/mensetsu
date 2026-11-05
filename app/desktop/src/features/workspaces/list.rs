use std::path::PathBuf;
use dioxus::prelude::*;
use mensetsu_contracts::models::WorkspaceResponseDto;
use crate::services::api::ApiService;
use crate::ui::test_ids::test_ids;

#[derive(Props, Clone, PartialEq)]
pub struct WorkspaceListProps {
    pub api: ApiService,
    pub on_select_workspace: EventHandler<(String, String)>,
    pub on_settings: EventHandler<()>,
    pub on_logout: EventHandler<()>,
}

#[component]
pub fn WorkspaceListView(props: WorkspaceListProps) -> Element {
    let mut workspaces = use_signal(Vec::<WorkspaceResponseDto>::new);
    let mut filter = use_signal(|| "all".to_string());
    let mut error_msg = use_signal(|| Option::<String>::None);
    let mut is_loading = use_signal(|| false);
    let mut show_create_dialog = use_signal(|| false);

    // Create dialog state
    let mut create_company = use_signal(String::new);
    let mut create_role = use_signal(String::new);
    let mut create_resume_text = use_signal(String::new);
    let mut create_jd = use_signal(String::new);
    let mut selected_pdf_path = use_signal(|| Option::<PathBuf>::None);
    let mut create_error = use_signal(|| Option::<String>::None);
    let mut is_creating = use_signal(|| false);

    let api = props.api.clone();
    let on_settings = props.on_settings;
    let on_logout = props.on_logout;
    let on_select_workspace = props.on_select_workspace;

    let load_workspaces = {
        let api = api.clone();
        Callback::new(move |()| {
            is_loading.set(true);
            error_msg.set(None);
            let api = api.clone();
        spawn(async move {
            match api.list_workspaces().await {
                Ok(list) => {
                    workspaces.set(list);
                }
                Err(e) => {
                    error_msg.set(Some(e.to_string()));
                }
            }
            is_loading.set(false);
            });
        })
    };

    use_effect(move || {
        load_workspaces.call(());
    });

    let handle_create_workspace = {
        let api = api.clone();
        let load_fn = load_workspaces;
        move |_| {
        let company = create_company.read().trim().to_string();
        let role = create_role.read().trim().to_string();
        let jd = create_jd.read().trim().to_string();
        let resume = create_resume_text.read().trim().to_string();
        let pdf_opt = selected_pdf_path.read().clone();

        if company.is_empty() {
            create_error.set(Some("Company name is required".to_string()));
            return;
        }
        if role.is_empty() {
            create_error.set(Some("Role title is required".to_string()));
            return;
        }
        if jd.is_empty() {
            create_error.set(Some("Job description is required".to_string()));
            return;
        }
        if resume.is_empty() && pdf_opt.is_none() {
            create_error.set(Some("Either resume text or a PDF resume file is required".to_string()));
            return;
        }

        create_error.set(None);
        is_creating.set(true);
            let api = api.clone();
            let load_fn = load_fn;
        spawn(async move {
            let res = if let Some(pdf_path) = pdf_opt {
                api.create_workspace_with_pdf(&company, &role, &jd, &pdf_path).await
            } else {
                api.create_workspace(&company, &role, &resume, &jd).await
            };

            is_creating.set(false);
            match res {
                Ok(_) => {
                    show_create_dialog.set(false);
                    create_company.set(String::new());
                    create_role.set(String::new());
                    create_resume_text.set(String::new());
                    create_jd.set(String::new());
                    selected_pdf_path.set(None);
                    load_fn.call(());
                }
                Err(e) => {
                    create_error.set(Some(e.to_string()));
                }
            }
        });
        }
    };

    let handle_pick_pdf = move |_| {
        let file = rfd::FileDialog::new()
            .add_filter("PDF files", &["pdf"])
            .pick_file();
        if let Some(path) = file {
            selected_pdf_path.set(Some(path));
            create_error.set(None);
        }
    };

    let cur_filter = filter.read().clone();
    let filtered_items = workspaces.read().iter().filter(|ws| {
        let status = format!("{}", ws.status);
        if cur_filter == "ready" {
            status.eq_ignore_ascii_case("ready") || status.eq_ignore_ascii_case("completed")
        } else if cur_filter == "processing" {
            status.eq_ignore_ascii_case("processing") || status.eq_ignore_ascii_case("running")
        } else {
            true
        }
    }).cloned().collect::<Vec<_>>();

    rsx! {
        div {
            class: "app-layout",
            id: "{test_ids::MAIN_VIEW}",
            "data-testid": "{test_ids::MAIN_VIEW}",
            // Header
            header { class: "app-header",
                h1 { class: "title-2", "Workspaces" }
                div { style: "display: flex; gap: 8px;",
                    button {
                        class: "primary",
                        id: "{test_ids::NEW_WORKSPACE_BUTTON}",
                        "data-testid": "{test_ids::NEW_WORKSPACE_BUTTON}",
                        onclick: move |_| show_create_dialog.set(true),
                        "+ New Workspace"
                    }
                    button {
                        class: "secondary",
                        id: "{test_ids::REFRESH_WORKSPACES_BUTTON}",
                        "data-testid": "{test_ids::REFRESH_WORKSPACES_BUTTON}",
                        onclick: move |_| load_workspaces.call(()),
                        "Refresh"
                    }
                    button {
                        class: "secondary",
                        id: "{test_ids::SETTINGS_BUTTON}",
                        "data-testid": "{test_ids::SETTINGS_BUTTON}",
                        onclick: move |_| on_settings.call(()),
                        "Settings"
                    }
                    button {
                        class: "destructive",
                        id: "{test_ids::LOGOUT_BUTTON}",
                        "data-testid": "{test_ids::LOGOUT_BUTTON}",
                        onclick: {
                            let api = api.clone();
                            move |_| {
                                let api = api.clone();
                                spawn(async move {
                                    let _ = api.logout().await;
                                    on_logout.call(());
                                });
                            }
                        },
                    }
                }
            }

            // Main content
            main { class: "app-content",
                // Filter bar
                div { style: "display: flex; gap: 8px; margin-bottom: 16px;",
                    button {
                        class: if cur_filter == "all" { "primary" } else { "secondary" },
                        id: "{test_ids::FILTER_ALL_BUTTON}",
                        "data-testid": "{test_ids::FILTER_ALL_BUTTON}",
                        onclick: move |_| filter.set("all".to_string()),
                        "All"
                    }
                    button {
                        class: if cur_filter == "ready" { "primary" } else { "secondary" },
                        id: "{test_ids::FILTER_READY_BUTTON}",
                        "data-testid": "{test_ids::FILTER_READY_BUTTON}",
                        onclick: move |_| filter.set("ready".to_string()),
                        "Ready"
                    }
                    button {
                        class: if cur_filter == "processing" { "primary" } else { "secondary" },
                        id: "{test_ids::FILTER_PROCESSING_BUTTON}",
                        "data-testid": "{test_ids::FILTER_PROCESSING_BUTTON}",
                        onclick: move |_| filter.set("processing".to_string()),
                        "Processing"
                    }
                }

                if let Some(err) = error_msg.read().as_ref() {
                    div { class: "error-banner", "{err}" }
                }

                if *is_loading.read() && filtered_items.is_empty() {
                    p { class: "dim-label", "Loading workspaces..." }
                } else if filtered_items.is_empty() {
                    p { class: "dim-label", "No workspaces found." }
                } else {
                    div {
                        class: "list-box",
                        id: "{test_ids::WORKSPACE_LIST}",
                        "data-testid": "{test_ids::WORKSPACE_LIST}",
                        for ws in filtered_items {
                            {
                                let ws_id = ws.id.clone();
                                let title = format!("{} — {}", ws.company_name, ws.role_title);
                                let item_test_id = format!("{}{}", test_ids::WORKSPACE_ITEM_PREFIX, ws.id);
                                let on_select = on_select_workspace;
                                rsx! {
                                    div {
                                        class: "list-item",
                                        key: "{ws.id}",
                                        id: "{item_test_id}",
                                        "data-testid": "{item_test_id}",
                                        onclick: move |_| on_select.call((ws_id.clone(), title.clone())),
                                        div {
                                            strong { style: "display: block; font-size: 15px; margin-bottom: 4px;", "{ws.company_name} — {ws.role_title}" }
                                            span { class: "dim-label", "ID: {ws.id}" }
                                        }
                                        div {
                                            span {
                                                style: "font-size: 12px; padding: 4px 8px; border-radius: 4px; background-color: var(--surface-border);",
                                                "{ws.status}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Create Workspace Modal
            if *show_create_dialog.read() {
                div { class: "modal-backdrop",
                    div { class: "modal",
                        h2 { class: "title-2", style: "margin-bottom: 16px;", "Create New Workspace" }

                        if let Some(err) = create_error.read().as_ref() {
                            div {
                                class: "error-banner",
                                id: "{test_ids::CREATE_ERROR_LABEL}",
                                "data-testid": "{test_ids::CREATE_ERROR_LABEL}",
                                "{err}"
                            }
                        }

                        div { style: "display: flex; flex-direction: column; gap: 12px; max-height: 65vh; overflow-y: auto; padding-right: 4px;",
                            div {
                                label { class: "dim-label", style: "display: block; margin-bottom: 4px;", "Company Name" }
                                input {
                                    id: "{test_ids::CREATE_COMPANY_INPUT}",
                                    "data-testid": "{test_ids::CREATE_COMPANY_INPUT}",
                                    placeholder: "e.g. Acme Corp",
                                    value: "{create_company}",
                                    oninput: move |evt| create_company.set(evt.value().clone()),
                                }
                            }

                            div {
                                label { class: "dim-label", style: "display: block; margin-bottom: 4px;", "Role Title" }
                                input {
                                    id: "{test_ids::CREATE_ROLE_INPUT}",
                                    "data-testid": "{test_ids::CREATE_ROLE_INPUT}",
                                    placeholder: "e.g. Senior Software Engineer",
                                    value: "{create_role}",
                                    oninput: move |evt| create_role.set(evt.value().clone()),
                                }
                            }

                            div {
                                label { class: "dim-label", style: "display: block; margin-bottom: 4px;", "Job Description" }
                                textarea {
                                    id: "{test_ids::CREATE_JD_INPUT}",
                                    "data-testid": "{test_ids::CREATE_JD_INPUT}",
                                    rows: "3",
                                    placeholder: "Paste the job description here...",
                                    value: "{create_jd}",
                                    oninput: move |evt| create_jd.set(evt.value().clone()),
                                }
                            }

                            div {
                                label { class: "dim-label", style: "display: block; margin-bottom: 4px;", "Résumé (Text or PDF)" }
                                textarea {
                                    id: "{test_ids::CREATE_RESUME_INPUT}",
                                    "data-testid": "{test_ids::CREATE_RESUME_INPUT}",
                                    rows: "3",
                                    placeholder: "Paste plain text résumé...",
                                    value: "{create_resume_text}",
                                    oninput: move |evt| create_resume_text.set(evt.value().clone()),
                                }
                                div { style: "display: flex; align-items: center; gap: 8px; margin-top: 6px;",
                                    button {
                                        class: "secondary",
                                        id: "{test_ids::CREATE_PDF_BUTTON}",
                                        "data-testid": "{test_ids::CREATE_PDF_BUTTON}",
                                        style: "font-size: 12px; padding: 4px 8px;",
                                        onclick: handle_pick_pdf,
                                        "Select PDF File"
                                    }
                                    if let Some(path) = selected_pdf_path.read().as_ref() {
                                        span {
                                            id: "{test_ids::CREATE_PDF_LABEL}",
                                            "data-testid": "{test_ids::CREATE_PDF_LABEL}",
                                            class: "dim-label",
                                            "{path.file_name().and_then(|n| n.to_str()).unwrap_or(\"selected.pdf\")}"
                                        }
                                    }
                                }
                            }
                        }

                        div { style: "display: flex; justify-content: flex-end; gap: 12px; margin-top: 20px;",
                            button {
                                class: "secondary",
                                id: "{test_ids::CREATE_CANCEL_BUTTON}",
                                "data-testid": "{test_ids::CREATE_CANCEL_BUTTON}",
                                onclick: move |_| show_create_dialog.set(false),
                                "Cancel"
                            }
                            button {
                                class: "primary",
                                id: "{test_ids::CREATE_SUBMIT_BUTTON}",
                                "data-testid": "{test_ids::CREATE_SUBMIT_BUTTON}",
                                disabled: *is_creating.read(),
                                onclick: handle_create_workspace,
                                if *is_creating.read() { "Creating..." } else { "Create" }
                            }
                        }
                    }
                }
            }
        }
    }
}
