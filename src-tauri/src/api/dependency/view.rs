use log::trace;

use super::core::NpmHandler;

#[tauri::command]
pub async fn check_npm() -> bool {
    let result = NpmHandler::detect().unwrap();
    trace!("check npm result: {}", result);
    result
}

#[tauri::command]
pub async fn install_npm() -> Result<(), String> {
    NpmHandler::install().map_err(|e| e.to_string())
}