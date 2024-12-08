mod api;

use log::{debug, info};
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_store::StoreExt;

pub const APP_STATE_FILENAME: &str = "AppState.json";

use api::dependency::view as dependency_view;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                    Target::new(TargetKind::Webview),
                ])
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            dependency_view::check_dependency,
            dependency_view::install_npm,
            dependency_view::install_uv
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
