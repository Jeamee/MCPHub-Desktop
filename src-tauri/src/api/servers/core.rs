use crate::utils::os::get_home;
use crate::APP_STATE_FILENAME;
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    #[serde(default)]
    guide: String,
    #[serde(rename = "isInstalled", default)]
    is_installed: bool,
    #[serde(default)]
    env: HashMap<String, String>,
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

impl ClientConfig {
    fn config_path() -> std::path::PathBuf {
        #[cfg(target_os = "macos")]
        {
        get_home()
            .unwrap()
            .join("Library/Application Support/Claude/claude_desktop_config.json")
        }
        #[cfg(target_os = "windows")]
        {
        let appdata = std::env::var("APPDATA").unwrap();
        std::path::PathBuf::from(appdata)
            .join("Claude")
            .join("claude_desktop_config.json")
        }
    }

    fn load() -> Self {
        let config_path = Self::config_path();
        let config = match std::fs::read_to_string(config_path) {
            Ok(content) => content,
            Err(_) => {
                debug!("Config file not found, returning empty HashMap");
                return ClientConfig {
                    mcp_servers: HashMap::new(),
                    other_fields: HashMap::new(),
                };
            }
        };
        debug!("ClientConfig loaded config");
        let config: ClientConfig = serde_json::from_str(&config).unwrap();
        debug!("ClientConfig parsed config");
        config
    }

    fn save(&self) {
        let config_path = Self::config_path();
        let config_str = serde_json::to_string_pretty(&self).unwrap();
        std::fs::write(config_path.clone(), config_str).unwrap();
    }
}


fn get_servers_from_store<T: for<'de> Deserialize<'de>>(app_handle: &tauri::AppHandle) -> Vec<T> {
    let store = app_handle.store(APP_STATE_FILENAME).unwrap();
    let raw_servers_str: String = serde_json::from_value(
        store
            .get("servers")
            .expect("Failed to get servers from store"),
    )
    .unwrap();
    let servers: Vec<T> = serde_json::from_str(&raw_servers_str).unwrap();
    servers
}

pub async fn get_client_server_config() -> HashMap<String, HashMap<String, String>> {
    debug!("get_client_server_config core");
    let config = ClientConfig::load();
    let mut id_env_map = HashMap::new();
    config
        .mcp_servers
        .iter()
        .for_each(|(title, server_config)| {
            id_env_map.insert(title.clone(), server_config.env.clone());
        });
    debug!("get_client_server_config core: loaded id_env_map");
    id_env_map
}

pub async fn load_all_frontend_servers(app_handle: &tauri::AppHandle) -> Vec<FrontendServer> {
    let mut servers = get_servers_from_store::<FrontendServer>(app_handle);
    debug!("load_all_frontend_servers core: loaded servers");
    let id_env_map = get_client_server_config().await;
    debug!("load_all_frontend_servers core: loaded id_env_map");
    servers.iter_mut().for_each(|server| {
        if id_env_map.contains_key(&server.base.id) {
            server.is_installed = true;
            server.env = id_env_map.get(&server.base.id).unwrap().clone();
        }
    });
    debug!("load_all_frontend_servers core: loaded servers");
    servers
}

pub async fn load_all_installed_frontend_servers(
    app_handle: &tauri::AppHandle,
) -> Vec<FrontendServer> {
    let servers = load_all_frontend_servers(app_handle).await;
    servers
        .into_iter()
        .filter(|server| server.is_installed)
        .collect()
}

pub async fn install_server_function(
    app_handle: &tauri::AppHandle,
    server_id: &str,
    env: Option<HashMap<String, String>>,
) -> bool {
    let mut servers = get_servers_from_store::<BackendServer>(app_handle);
    let server = servers
        .iter_mut()
        .find(|server| server.base.id == server_id)
        .unwrap();
    let mut command = server.command_info.command.clone();
    let mut args = server.command_info.args.clone();
    let env = env.unwrap_or_else(|| server.command_info.env.clone());

    let mut config = ClientConfig::load();
    let store = app_handle.store(APP_STATE_FILENAME).unwrap();

    if command == "npx" {
        let use_system_node = store
            .get("use_system_node")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
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
        let use_system_uv = store
            .get("use_system_uv")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
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

    config.mcp_servers.insert(
        server_id.to_string(),
        ClientServerConfig { command, args, env },
    );
    config.save();
    true
}



pub async fn uninstall_server_function(server_id: &str) -> bool {
    let mut config = ClientConfig::load();
    config.mcp_servers.remove(&server_id.to_string());
    config.save();
    true
}

pub async fn update_server_function(
    app_handle: &tauri::AppHandle,
    server_id: &str,
    env: HashMap<String, String>,
) -> bool {
    let mut config = ClientConfig::load();
    if !config.mcp_servers.contains_key(server_id) {
        install_server_function(&app_handle, server_id, Some(env)).await;
    } else {
        let server_config = config.mcp_servers.get_mut(server_id).unwrap();
        server_config.env = env;
        config.save();
    }
    true
}
