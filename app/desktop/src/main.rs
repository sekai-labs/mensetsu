#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use dioxus::desktop::{Config, WindowBuilder};
use mensetsu_desktop::App;

fn main() {
    let window = WindowBuilder::new()
        .with_title("Mensetsu — Technical Interview Preparation")
        .with_inner_size(dioxus::desktop::LogicalSize::new(1080.0, 780.0))
        .with_min_inner_size(dioxus::desktop::LogicalSize::new(800.0, 600.0));

    let cfg = Config::new().with_window(window);

    dioxus::LaunchBuilder::desktop()
        .with_cfg(cfg)
        .launch(App);
}
