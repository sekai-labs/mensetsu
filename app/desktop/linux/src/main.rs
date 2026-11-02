use gtk4::prelude::*;
use gtk4::Application;
use std::env;

mod api_service;
mod app_root;
mod secure_store;
mod test_ids;
pub mod ui;

use api_service::ApiService;
use app_root::AppRoot;

const APP_ID: &str = "com.sekailabs.mensetsu.linux";

pub fn get_api_base_url() -> String {
    env::var("MENSETSU_API_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".to_string())
}

fn build_ui(app: &Application) {
    let base_url = get_api_base_url();
    let api = ApiService::new(base_url);
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
