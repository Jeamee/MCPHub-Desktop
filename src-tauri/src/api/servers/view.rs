use super::core::{
    install_server_function, load_all_frontend_servers, load_all_installed_frontend_servers,
    uninstall_server_function, update_server_function, FrontendServer,
};
use log::debug;
use std::collections::HashMap;
use tauri_plugin_store::StoreExt;

#[tauri::command]
pub async fn get_servers(app_handle: tauri::AppHandle) -> Vec<FrontendServer> {
    debug!("get_servers view");
    load_all_frontend_servers(&app_handle).await
}

#[tauri::command]
pub async fn get_installed_servers(app_handle: tauri::AppHandle) -> Vec<FrontendServer> {
    load_all_installed_frontend_servers(&app_handle).await
}

#[tauri::command]
pub async fn install_server(app_handle: tauri::AppHandle, server_id: &str) -> Result<bool, String> {
    Ok(install_server_function(&app_handle, server_id, None).await)
}

#[tauri::command]
pub async fn update_server(
    app_handle: tauri::AppHandle,
    server_id: &str,
    env: Option<HashMap<String, String>>,
) -> Result<bool, String> {
    Ok(update_server_function(&app_handle, server_id, env.unwrap_or_default()).await)
}

#[tauri::command]
pub async fn uninstall_server(server_id: &str) -> Result<bool, String> {
    Ok(uninstall_server_function(server_id).await)
}
