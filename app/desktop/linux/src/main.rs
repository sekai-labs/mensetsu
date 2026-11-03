use gtk4::prelude::*;
use gtk4::Application;

mod api_service;
mod endpoint_config;
mod app_root;
mod secure_store;
mod test_ids;
pub mod ui;

use api_service::ApiService;
use app_root::AppRoot;

const APP_ID: &str = "com.sekailabs.mensetsu.linux";

fn build_ui(app: &Application) {
    // If a saved config exists, use it; otherwise start unconfigured with empty placeholder.
    // AppRoot detects if endpoint is configured; if not, it presents ServerSetupView immediately.
    let initial_base_url = endpoint_config::resolve_initial_url().unwrap_or_default();
    let api = ApiService::new(initial_base_url);
    let root = AppRoot::new(api);
    root.window.set_application(Some(app));
    root.window.present();
}

fn main() -> glib::ExitCode {
    // GTK owns the main thread; entering a Tokio runtime lets its GLib futures
    // use reqwest and Secret Service without blocking the event loop.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Tokio runtime required for network access");
    let _runtime_guard = runtime.enter();
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run()
}
