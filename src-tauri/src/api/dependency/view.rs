use serde::{Deserialize, Serialize};
use tauri_plugin_store::StoreExt;

use super::core::{NpmHandler, UVHandler, ResourceHandler};

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyStatus {
    uv: bool,
    node: bool,
}

#[tauri::command]
pub async fn check_dependency(app_handle: tauri::AppHandle) -> DependencyStatus {
    let status = DependencyStatus {
        uv: UVHandler::detect(&app_handle).unwrap_or(false),
        node: NpmHandler::detect(&app_handle).unwrap_or(false),
    };
    status
}

#[tauri::command]
pub async fn install_npm(app_handle: tauri::AppHandle) -> Result<(), String> {
    NpmHandler::install(&app_handle).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn install_uv(app_handle: tauri::AppHandle) -> Result<(), String> {
    UVHandler::install(&app_handle).await.map_err(|e| e.to_string())
}


#[tauri::command]
pub async fn check_resource(app_handle: tauri::AppHandle) -> bool {
    ResourceHandler::detect(&app_handle).await.unwrap_or(false)
}
