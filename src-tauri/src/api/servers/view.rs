use super::core::{FrontendServer, load_all_frontend_servers, load_all_installed_frontend_servers, install_server_function, uninstall_server_function};
use tauri_plugin_store::StoreExt;


#[tauri::command]
pub async fn get_servers(app_handle: tauri::AppHandle) -> Result<Vec<FrontendServer>, String> {
    Ok(load_all_frontend_servers(&app_handle))
}

#[tauri::command]
pub async fn get_installed_servers(app_handle: tauri::AppHandle) -> Result<Vec<FrontendServer>, String> {
    Ok(load_all_installed_frontend_servers(&app_handle))
}

#[tauri::command]
pub async fn install_server(app_handle: tauri::AppHandle, server_id: &str) -> Result<bool, String> {
    Ok(install_server_function(&app_handle, server_id))
}

#[tauri::command]
pub async fn uninstall_server(server_id: &str) -> Result<bool, String> {
    Ok(uninstall_server_function(server_id))
}