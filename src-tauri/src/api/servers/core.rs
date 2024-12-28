use crate::utils::os::get_home;
use std::borrow::Cow;
use crate::APP_STATE_FILENAME;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri_plugin_store::StoreExt;
use shell_escape::escape;

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
    #[serde(default)]
    env: HashMap<String, String>,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    guide: String,
    #[serde(default, rename = "inputArg")]
    input_arg: InputArg,
}

#[derive(Debug, Serialize, Deserialize)]
struct SystemCommandInfo {
    command: String,
    args: Vec<String>,
    #[serde(default, rename = "inputArg")]
    input_arg: InputArg,
    #[serde(default)]
    env: HashMap<String, String>,
    #[serde(default)]
    guide: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackendServer {
    #[serde(flatten)]
    base: BaseServer,
    #[serde(rename = "commandInfo")]
    command_info: SystemCommandInfo,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
enum ArgClass {
    #[default]
    Text,
    Select,
    FilePath,
    DirectoryPath,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
enum ArgumentMultiplicity {
    #[default]
    Single,
    Multiple,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct InputArg {
    #[serde(default)]
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    class: ArgClass,
    #[serde(default)]
    multiplicity: ArgumentMultiplicity,
    #[serde(default)]
    value: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientServerConfig {
    #[serde(default)]
    command: String,
    args: Vec<String>,
    #[serde(default)]
    env: HashMap<String, String>,
    #[serde(rename = "commandCreator", default)]
    command_creator: String,
    #[serde(rename = "inputArg", default)]
    input_arg: InputArg,
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

pub async fn get_client_server_config() -> HashMap<String, ClientServerConfig> {
    debug!("get_client_server_config core");
    let config = ClientConfig::load();
    let mut id_config_map = HashMap::new();
    config
        .mcp_servers
        .into_iter()
        .for_each(|(title, server_config)| {
            id_config_map.insert(title, server_config);
        });
    debug!("get_client_server_config core: loaded id_env_map");
    id_config_map
}

pub async fn load_all_frontend_servers(app_handle: &tauri::AppHandle) -> Vec<FrontendServer> {
    let backend_servers = get_servers_from_store::<BackendServer>(app_handle);
    debug!("load_all_frontend_servers core: loaded servers");
    let id_config_map = get_client_server_config().await;
    debug!("load_all_frontend_servers core: loaded id_env_map");

    backend_servers
        .into_iter()
        .map(|mut backend_server| {
            let is_installed = id_config_map.contains_key(&backend_server.base.id);
            let env = if is_installed {
                id_config_map
                    .get(&backend_server.base.id)
                    .unwrap()
                    .env
                    .clone()
            } else {
                backend_server.command_info.env
            };

            let arg_values = if is_installed {
                id_config_map
                    .get(&backend_server.base.id)
                    .unwrap()
                    .input_arg
                    .value
                    .clone()
            } else {
                vec![]
            };

            backend_server.command_info.input_arg.value = arg_values;

            FrontendServer {
                base: backend_server.base,
                is_installed,
                env,
                guide: backend_server.command_info.guide,
                args: backend_server.command_info.args,
                input_arg: backend_server.command_info.input_arg,
            }
        })
        .collect()
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
    input_arg: Option<Vec<String>>,
) -> bool {
    let mut servers = get_servers_from_store::<BackendServer>(app_handle);
    let server = servers
        .iter_mut()
        .find(|server| server.base.id == server_id)
        .unwrap();
    let mut command = server.command_info.command.clone();
    let mut arg_configs = server.command_info.args.join(" ");
    let mut input_arg_config = server.command_info.input_arg.clone();
    let env = env.unwrap_or_else(|| server.command_info.env.clone());
    if input_arg.is_some() {
        input_arg_config.value = input_arg.unwrap();
        arg_configs = format!("{} {}", arg_configs, escape(Cow::from(input_arg_config.value.join(" "))));
    }

    let mut config = ClientConfig::load();
    let store = app_handle.store(APP_STATE_FILENAME).unwrap();
    let mut args = vec![];

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
                    format!("PATH=\"{}:$PATH\" npx {}", node_path, arg_configs),
                ];
            }
            #[cfg(target_os = "windows")]
            {
                command = "cmd".to_string();
                args = vec![
                    "/c".to_string(),
                    format!("set PATH=%PATH%;{} && npx {}", node_path, arg_configs),
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
                    format!("PATH=\"{}:$PATH\" uvx {}", uv_path, arg_configs),
                ];
            }
            #[cfg(target_os = "windows")]
            {
                command = "cmd".to_string();
                args = vec![
                    "/c".to_string(),
                    format!("set PATH=%PATH%;{} && uvx {}", uv_path, arg_configs),
                ];
            }
        }
    }

    config.mcp_servers.insert(
        server_id.to_string(),
        ClientServerConfig {
            command,
            args,
            env,
            command_creator: "MCPHub".to_string(),
            input_arg: input_arg_config,
        },
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
    env: Option<HashMap<String, String>>,
    input_arg: Option<Vec<String>>,
) -> bool {
    let mut config = ClientConfig::load();
    if !config.mcp_servers.contains_key(server_id) {
        install_server_function(&app_handle, server_id, env, input_arg).await;
    } else {
        uninstall_server_function(server_id).await;
        install_server_function(&app_handle, server_id, env, input_arg).await;
    }
    true
}
