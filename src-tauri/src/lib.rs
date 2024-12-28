mod api;
mod utils;

use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_store::StoreExt;

pub const APP_STATE_FILENAME: &str = "AppState.json";

use api::dependency::view as dependency_view;
use api::servers::view as servers_view;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                ])
                .build(),
        )
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            dependency_view::check_dependency,
            dependency_view::check_resource,
            dependency_view::install_npm,
            dependency_view::install_uv,
            servers_view::get_servers,
            servers_view::get_installed_servers,
            servers_view::install_server,
            servers_view::uninstall_server,
            servers_view::update_server,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
