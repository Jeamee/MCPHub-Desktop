use super::core::{FrontendServer, load_all_frontend_servers, load_all_installed_frontend_servers};
use tauri_plugin_store::StoreExt;


#[tauri::command]
pub async fn get_servers(app_handle: tauri::AppHandle) -> Result<Vec<FrontendServer>, String> {
    Ok(load_all_frontend_servers(&app_handle))
}

#[tauri::command]
pub async fn get_installed_servers(app_handle: tauri::AppHandle) -> Result<Vec<FrontendServer>, String> {
    Ok(load_all_installed_frontend_servers(&app_handle))
}
