use gtk4::prelude::*;
use gtk4::{Stack, StackTransitionType, Window};
use std::rc::Rc;

use crate::api_service::ApiService;
use crate::endpoint_config::resolve_initial_url;
use crate::ui::auth_view::AuthView;
use crate::ui::server_setup_view::ServerSetupView;
use crate::ui::settings_view::SettingsView;
use crate::ui::workspace_detail_view::WorkspaceDetailView;
use crate::ui::workspace_list_view::WorkspaceListView;
pub struct AppRoot {
    pub window: Window,
}

impl AppRoot {
    pub fn new(api: ApiService) -> Self {
        let window = Window::builder()
            .title("Mensetsu — Technical Interview Preparation")
            .default_width(1000)
            .default_height(750)
            .build();

        let stack = Stack::new();
        stack.set_transition_type(StackTransitionType::Crossfade);

        let stack_rc = Rc::new(stack.clone());
        let api_rc = Rc::new(api.clone());

        let show_workspace_list = {
            let stack = stack_rc.clone();
            let api = api_rc.clone();

            Rc::new(move || {
                if let Some(child) = stack.child_by_name("main") {
                    stack.remove(&child);
                }

                let list_view = WorkspaceListView::new(
                    (*api).clone(),
                    {
                        let stack = stack.clone();
                        let api = api.clone();
                        move |ws_id, ws_title| {
                            if let Some(old_detail) = stack.child_by_name("detail") {
                                stack.remove(&old_detail);
                            }
                            let stack_back = stack.clone();
                            let detail_view = WorkspaceDetailView::new(
                                (*api).clone(),
                                ws_id,
                                ws_title,
                                move || {
                                    stack_back.set_visible_child_name("main");
                                },
                            );
                            stack.add_named(&detail_view.container, Some("detail"));
                            stack.set_visible_child_name("detail");
                        }
                    },
                    {
                        let stack = stack.clone();
                        let api = api.clone();
                        move || {
                            if let Some(old_settings) = stack.child_by_name("settings") {
                                stack.remove(&old_settings);
                            }
                            let stack_back = stack.clone();
                            let stack_reset = stack.clone();
                            let settings_view = SettingsView::new(
                                (*api).clone(),
                                move || {
                                    stack_back.set_visible_child_name("main");
                                },
                                move || {
                                    // When server is changed, session is cleared, route back to auth
                                    stack_reset.set_visible_child_name("auth");
                                },
                            );
                            stack.add_named(&settings_view.container, Some("settings"));
                            stack.set_visible_child_name("settings");
                        }
                    },
                    {
                        let stack = stack.clone();
                        move || {
                            stack.set_visible_child_name("auth");
                        }
                    },
                );

                stack.add_named(&list_view.container, Some("main"));
                stack.set_visible_child_name("main");
            })
        };

        let initial_config = resolve_initial_url();
        let has_configured_endpoint = initial_config.is_some();

        let show_auth = {
            let stack = stack_rc.clone();
            let api = api_rc.clone();
            let show_list = show_workspace_list.clone();

            Rc::new(move || {
                if let Some(old_auth) = stack.child_by_name("auth") {
                    stack.remove(&old_auth);
                }
                let stack_for_change = stack.clone();
                let api_for_change = api.clone();
                let show_list_cb = show_list.clone();

                let auth_view = AuthView::new(
                    (*api).clone(),
                    move || {
                        show_list_cb();
                    },
                    move || {
                        // Pre-auth change server button clicked
                        if let Some(old_setup) = stack_for_change.child_by_name("setup") {
                            stack_for_change.remove(&old_setup);
                        }
                        let stack_setup = stack_for_change.clone();
                        let stack_cancel = stack_for_change.clone();
                        let api_setup = api_for_change.clone();
                        let current = Some(api_setup.get_base_url());
                        let setup_view = ServerSetupView::new(
                            (*api_setup).clone(),
                            current,
                            move |_new_url| {
                                stack_setup.set_visible_child_name("auth");
                            },
                            Some(move || {
                                stack_cancel.set_visible_child_name("auth");
                            }),
                        );
                        stack_for_change.add_named(&setup_view.container, Some("setup"));
                        stack_for_change.set_visible_child_name("setup");
                    },
                );
                stack.add_named(&auth_view.container, Some("auth"));
            })
        };

        show_auth();

        if !has_configured_endpoint {
            // First launch: no saved endpoint
            // Prompt user for URL before allowing any auth/API calls
            let stack_setup = stack_rc.clone();
            let api_setup = api_rc.clone();
            let show_auth_cb = show_auth.clone();
            let setup_view = ServerSetupView::new(
                (*api_setup).clone(),
                None,
                move |_new_url| {
                    show_auth_cb();
                    stack_setup.set_visible_child_name("auth");
                },
                Option::<fn()>::None,
            );
            stack_rc.add_named(&setup_view.container, Some("setup"));
            stack_rc.set_visible_child_name("setup");
        } else {
            // Configured endpoint exists: check login state
            let api = api_rc.clone();
            let show_list = show_workspace_list.clone();
            let stack = stack_rc.clone();

            glib::MainContext::default().spawn_local(async move {
                if api.is_logged_in().await {
                    if api.refresh_access_token().await.is_ok() {
                        show_list();
                        return;
                    }
                }
                stack.set_visible_child_name("auth");
            });
        }

        window.set_child(Some(&stack));

        Self {
            window,
        }
    }
}
