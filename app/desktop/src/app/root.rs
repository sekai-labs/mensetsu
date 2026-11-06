use dioxus::prelude::*;
use crate::features::{
    auth::view::AuthView,
    server_setup::view::ServerSetupView,
    settings::view::SettingsView,
    workspaces::{detail::WorkspaceDetailView, list::WorkspaceListView},
};
use crate::services::{api::ApiService, endpoint_config::resolve_initial_url};
use crate::ui::styles::APP_CSS;

#[derive(Clone, PartialEq)]
pub enum CurrentScreen {
    Setup { current_url: Option<String>, on_cancel_allowed: bool },
    Auth,
    WorkspaceList,
    WorkspaceDetail { id: String, title: String },
    Settings,
}

#[component]
pub fn App() -> Element {
    let initial_url = resolve_initial_url();
    let has_endpoint = initial_url.is_some();
    let initial_base = initial_url.clone().unwrap_or_default();
    let api = use_signal(|| ApiService::new(initial_base));

    let mut current_screen = use_signal(|| {
        if has_endpoint {
            CurrentScreen::Auth
        } else {
            CurrentScreen::Setup { current_url: None, on_cancel_allowed: false }
        }
    });


    use_effect({
        let api_sig = api.clone();
        move || {
            if has_endpoint {
                let api = api_sig.read().clone();
                spawn(async move {
                    if api.is_logged_in().await {
                        if api.refresh_access_token().await.is_ok() {
                            current_screen.set(CurrentScreen::WorkspaceList);
                        }
                    }
                });
            }
        }
    });

    let screen = match current_screen.read().clone() {
        CurrentScreen::Setup { current_url, on_cancel_allowed } => rsx! {
            ServerSetupView {
                api: api.read().clone(),
                current_url: current_url,
                on_saved: move |_| current_screen.set(CurrentScreen::Auth),
                on_cancel: if on_cancel_allowed {
                    Some(EventHandler::new(move |_| current_screen.set(CurrentScreen::Auth)))
                } else {
                    None
                },
            }
        },
        CurrentScreen::Auth => rsx! {
            AuthView {
                api: api.read().clone(),
                on_success: move |_| current_screen.set(CurrentScreen::WorkspaceList),
                on_change_server: move |_| {
                    let cur_url = Some(api.read().get_base_url());
                    current_screen.set(CurrentScreen::Setup { current_url: cur_url, on_cancel_allowed: true });
                },
            }
        },
        CurrentScreen::WorkspaceList => rsx! {
            WorkspaceListView {
                api: api.read().clone(),
                on_select_workspace: move |(id, title)| {
                    current_screen.set(CurrentScreen::WorkspaceDetail { id, title });
                },
                on_settings: move |_| current_screen.set(CurrentScreen::Settings),
                on_logout: move |_| current_screen.set(CurrentScreen::Auth),
            }
        },
        CurrentScreen::WorkspaceDetail { id, title } => rsx! {
            WorkspaceDetailView {
                api: api.read().clone(),
                workspace_id: id,
                workspace_title: title,
                on_back: move |_| current_screen.set(CurrentScreen::WorkspaceList),
            }
        },
        CurrentScreen::Settings => rsx! {
            SettingsView {
                api: api.read().clone(),
                on_back: move |_| current_screen.set(CurrentScreen::WorkspaceList),
                on_server_changed: move |_| current_screen.set(CurrentScreen::Auth),
            }
        },
    };

    rsx! {
        style { "{APP_CSS}" }
        {screen}
    }
}
