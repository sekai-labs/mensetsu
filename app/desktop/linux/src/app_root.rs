use gtk4::prelude::*;
use gtk4::{Stack, StackTransitionType, Window};
use std::rc::Rc;

use crate::api_service::ApiService;
use crate::ui::auth_view::AuthView;
use crate::ui::workspace_list_view::WorkspaceListView;
use crate::ui::workspace_detail_view::WorkspaceDetailView;
use crate::ui::settings_view::SettingsView;

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
                            let settings_view = SettingsView::new(
                                (*api).clone(),
                                move || {
                                    stack_back.set_visible_child_name("main");
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

        let auth_view = {
            let show_list = show_workspace_list.clone();
            AuthView::new((*api_rc).clone(), move || {
                show_list();
            })
        };
        stack.add_named(&auth_view.container, Some("auth"));

        // Check login state
        {
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
