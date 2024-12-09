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
struct CommandInfo {
    #[serde(rename = "command")]
    command: String,
    args: Vec<String>,
    #[serde(default)]
    env: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackendServer {
    #[serde(flatten)]
    base: BaseServer,
    #[serde(rename = "commandInfo")]
    command_info: CommandInfo,

}

#[derive(Debug, Serialize, Deserialize)]
struct ClientServerConfig {
    #[serde(default)]
    command: String,
    args: Vec<String>,
    #[serde(default)]
    env: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientConfig {
    #[serde(rename = "mcpServers", default)]
    mcp_servers: HashMap<String, ClientServerConfig>,
    #[serde(flatten, default)]
    other_fields: HashMap<String, serde_json::Value>,
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

pub fn install_server_function(app_handle: &tauri::AppHandle, server_id: &str) -> bool {
    let store = app_handle.store(APP_STATE_FILENAME).unwrap();
    let raw_servers_str: String = serde_json::from_value(store.get("servers").expect("Failed to get servers from store")).unwrap();
    let mut servers: Vec<BackendServer> = serde_json::from_str(&raw_servers_str).unwrap();
    let server = servers.iter_mut().find(|server| server.base.id == server_id).unwrap();
    let mut command = server.command_info.command.clone();
    let mut args = server.command_info.args.clone();
    let env = server.command_info.env.clone();

    #[cfg(target_os = "macos")]
    let config_path = get_home().unwrap().join("Library/Application Support/Claude/claude_desktop_config.json");
    #[cfg(target_os = "windows")]
    let config_path = get_home().unwrap().join(std::env::var("APPDATA").unwrap()).join("Claude/claude_desktop_config.json");
    let config = std::fs::read_to_string(config_path.clone()).unwrap();
    let mut config: ClientConfig = serde_json::from_str(&config).unwrap();

    if command == "npx" {
        let use_system_node = store.get("use_system_node").and_then(|v| v.as_bool()).unwrap_or(false);
        let node_path = store
            .get("node_path")
            .and_then(|s| s.as_str().map(String::from))
            .unwrap_or("".to_owned());
        if !use_system_node {
            #[cfg(target_os = "macos")]
            {
                command = "sh".to_string();
                args = vec![
                    "-c".to_string(),
                    format!("PATH=\"{}:$PATH\" npx {}", node_path, args.join(" ")),
                ];
            }
            #[cfg(target_os = "windows")]
            {
                command = "cmd".to_string();
                args = vec![
                    "/c".to_string(),
                    format!("set PATH=%PATH%;{} && npx {}", node_path, args.join(" ")),
                ];
            }
        }
    } else if command == "uvx" {
        let use_system_uv = store.get("use_system_uv").and_then(|v| v.as_bool()).unwrap_or(false);
        let uv_path = store
            .get("uv_path")
            .and_then(|s| s.as_str().map(String::from))
            .unwrap_or("".to_owned());
        if !use_system_uv {
            #[cfg(target_os = "macos")]
            {
                command = "sh".to_string();
                args = vec![
                    "-c".to_string(),
                    format!("PATH=\"{}:$PATH\" uvx {}", uv_path, args.join(" ")),
                ];
            }
            #[cfg(target_os = "windows")]
            {
                command = "cmd".to_string();
                args = vec![
                    "/c".to_string(),
                    format!("set PATH=%PATH%;{} && uvx {}", uv_path, args.join(" ")),
                ];
            }
        }
    }

    config.mcp_servers.insert(server_id.to_string(), ClientServerConfig {
        command,
        args,
        env,
    });
    let config_str = serde_json::to_string_pretty(&config).unwrap();
    std::fs::write(config_path.clone(), config_str).unwrap();

    true
}


pub fn uninstall_server_function(server_id: &str) -> bool {
    let config_path = get_home().unwrap().join("Library/Application Support/Claude/claude_desktop_config.json");
    let config = std::fs::read_to_string(config_path.clone()).unwrap();
    let mut config: ClientConfig = serde_json::from_str(&config).unwrap();
    config.mcp_servers.remove(&server_id.to_string());
    let config_str = serde_json::to_string_pretty(&config).unwrap();
    std::fs::write(config_path.clone(), config_str).unwrap();
    true
}