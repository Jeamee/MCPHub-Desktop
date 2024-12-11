use serde::{Deserialize, Serialize};
use log::debug;
use tauri_plugin_store::StoreExt;

use super::core::{NpmHandler, ResourceHandler, UVHandler};

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyStatus {
    uv: bool,
    node: bool,
    servers: bool,
}

#[tauri::command]
pub async fn check_dependency(app_handle: tauri::AppHandle) -> DependencyStatus {
    let status = DependencyStatus {
        uv: UVHandler::detect(&app_handle).await.unwrap_or(false),
        node: NpmHandler::detect(&app_handle).await.unwrap_or(false),
        servers: ResourceHandler::detect(&app_handle).await.unwrap_or(false),
    };
    status
}

#[tauri::command]
pub async fn install_npm(app_handle: tauri::AppHandle) -> Result<(), String> {
    NpmHandler::install(&app_handle)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn install_uv(app_handle: tauri::AppHandle) -> Result<(), String> {
    UVHandler::install(&app_handle)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_resource(app_handle: tauri::AppHandle) -> bool {
    debug!("Start check_resource in backend");
    ResourceHandler::detect(&app_handle).await.unwrap_or(false)
}
