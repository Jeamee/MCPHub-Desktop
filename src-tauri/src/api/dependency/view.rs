use log::{error, trace};

use super::core::{NpmHandler, UVHandler};

#[tauri::command]
pub async fn check_npm() -> bool {
    match NpmHandler::detect() {
        Ok(result) => {
            trace!("check npm result: {}", result);
            result
        }
        Err(e) => {
            error!("Failed to detect npm: {}", e);
            false
        }
    }
}

#[tauri::command]
pub async fn install_npm() -> Result<(), String> {
    NpmHandler::install().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_uv() -> bool {
    match UVHandler::detect() {
        Ok(result) => {
            trace!("check uv result: {}", result);
            result
        }
        Err(e) => {
            error!("Failed to detect uv: {}", e);
            false
        }
    }
}

#[tauri::command]
pub async fn install_uv() -> Result<(), String> {
    UVHandler::install().await.map_err(|e| e.to_string())
}