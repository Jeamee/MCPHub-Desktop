use log::debug;
use crate::utils::os::get_home;
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use crate::APP_STATE_FILENAME;
use tauri_plugin_store::StoreExt;

#[derive(Debug, Serialize, Deserialize)]
struct BaseServer {
    id: String,
    title: String,
    description: String,
    creator: String,
    tags: Vec<String>,
    #[serde(rename = "logoUrl")]
    logo_url: String,
    rating: u8,
    #[serde(rename = "publishDate")]
    publish_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FrontendServer {
    #[serde(flatten)]
    base: BaseServer,
    #[serde(rename = "isInstalled", default)]
    is_installed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClientServerConfig {
    #[serde(default)]
    command: String,
    args: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientConfig {
    #[serde(rename = "mcpServers", default)]
    mcp_servers: HashMap<String, ClientServerConfig>,
}

pub fn get_client_server_identifiers() -> HashSet<String> {
    let config_path = get_home().unwrap().join("Library/Application Support/Claude/claude_desktop_config.json");
    let config = std::fs::read_to_string(config_path).unwrap();
    let config: ClientConfig = serde_json::from_str(&config).unwrap();
    let mut identifiers = HashSet::new();
    config.mcp_servers.iter().for_each(|(title, _)| {
        identifiers.insert(title.clone());
    });
    identifiers
}

pub fn load_all_frontend_servers(app_handle: &tauri::AppHandle) -> Vec<FrontendServer> {
    let store = app_handle.store(APP_STATE_FILENAME).unwrap();
    let raw_servers_str: String = serde_json::from_value(store.get("servers").expect("Failed to get servers from store")).unwrap();
    let mut servers: Vec<FrontendServer> = serde_json::from_str(&raw_servers_str).unwrap();
    let existing_identifiers = get_client_server_identifiers();
    servers.iter_mut().for_each(|server| {
        if existing_identifiers.contains(&server.base.id) {
            server.is_installed = true;
        }
    });
    servers
}

pub fn load_all_installed_frontend_servers(app_handle: &tauri::AppHandle) -> Vec<FrontendServer> {
    let servers = load_all_frontend_servers(app_handle);
    servers.into_iter().filter(|server| server.is_installed).collect()
}
