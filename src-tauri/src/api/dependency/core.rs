use crate::APP_STATE_FILENAME;
use anyhow::{anyhow, Context, Result};
#[cfg(target_os = "macos")]
use flate2::read::GzDecoder;
use log::{debug, error, trace};
use reqwest;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::io::{Cursor, Read};
use std::path::PathBuf;
#[cfg(target_os = "macos")]
use tar::Archive;
use tauri_plugin_store::StoreExt;
use xshell::{cmd, Shell};
#[cfg(target_os = "windows")]
use zip::ZipArchive;

use crate::utils::os::{detect_shell, get_home};

pub struct NpmHandler;
pub struct UVHandler;

pub struct ResourceHandler;
const SERVERS_URL: &str = "https://app.mcphub.net/server-configuration/servers.json";

impl NpmHandler {
    pub fn detect(app_handle: &tauri::AppHandle) -> Result<bool> {
        let store = app_handle.store(APP_STATE_FILENAME).unwrap();
        let shell = Shell::new()?;
        let shell_name = detect_shell()?;

        let node_path = store
            .get("node_path")
            .and_then(|s| s.as_str().map(String::from))
            .unwrap_or("".to_owned());
        if (!node_path.is_empty()) {
            if let Ok(metadata) = fs::metadata(&node_path) {
                if metadata.is_dir() || metadata.is_symlink() {
                    debug!("Node path exists: {}", node_path);
                    return Ok(true);
                }
            }
            debug!("Node path does not exist: {}", node_path);
        }
        debug!("Running check node command");

        #[cfg(target_os = "macos")]
        let cmd_output = cmd!(shell, "{shell_name} -ic 'which node'")
            .quiet()
            .read()?;

        #[cfg(target_os = "windows")]
        let cmd_output = cmd!(shell, "where.exe node").quiet().read()?;

        debug!("Node command output: {}", cmd_output);
        store.set("node_path", node_path);
        store.set("use_system_node", true);

        Ok(true)
    }

    pub async fn install(app_handle: &tauri::AppHandle) -> Result<()> {
        trace!("Installing Node.js");
        let store = app_handle.store(APP_STATE_FILENAME)?;
        let shell = Shell::new()?;
        let home_dir_str = get_home()?.to_string_lossy().to_string();

        #[cfg(target_os = "macos")]
        let (node_version, node_arch, node_dir_name, node_download_url) = {
            let node_version = "v22.11.0";
            let node_arch = if cfg!(target_arch = "aarch64") {
                "darwin-arm64"
            } else {
                "darwin-x64"
            };
            let node_dir_name = format!("node-{}-{}", node_version, node_arch);
            let node_download_url = format!(
                "https://nodejs.org/dist/{}/node-{}-{}.tar.gz",
                node_version, node_version, node_arch
            );
            (node_version, node_arch, node_dir_name, node_download_url)
        };

        #[cfg(target_os = "windows")]
        let (node_version, node_arch, node_dir_name, node_download_url) = {
            let node_version = "v22.11.0";
            let node_arch = if cfg!(target_arch = "x86_64") {
                "win-x64"
            } else {
                "win-x86"
            };
            let node_dir_name = format!("node-{}-{}", node_version, node_arch);
            let node_download_url = format!(
                "https://nodejs.org/dist/{}/node-{}-{}.zip",
                node_version, node_version, node_arch
            );
            (node_version, node_arch, node_dir_name, node_download_url)
        };

        trace!("Downloading node from {}", node_download_url);

        // Create .node directory using fs
        #[cfg(target_os = "macos")]
        let node_dir = format!("{}/.node", home_dir_str);
        #[cfg(target_os = "windows")]
        let node_dir = format!("{}\\AppData\\Local\\node", home_dir_str);

        trace!("Creating node directory at {}", node_dir);
        fs::create_dir_all(&node_dir)?;

        // Download using reqwest async
        trace!("Downloading node.js");
        let response = reqwest::get(node_download_url).await?;
        let bytes = response.bytes().await?;

        // Extract archive
        trace!("Extracting archive");
        #[cfg(target_os = "macos")]
        {
            let gz = GzDecoder::new(Cursor::new(bytes));
            let mut archive = Archive::new(gz);
            archive.unpack(&node_dir)?;
        }

        #[cfg(target_os = "windows")]
        {
            let cursor = Cursor::new(bytes);
            let mut archive = ZipArchive::new(cursor)?;
            archive.extract(&node_dir)?;
        }

        store.set("node_path", node_dir);
        store.set("use_system_node", false);
        trace!("All done");
        Ok(())
    }
}

impl UVHandler {
    pub fn detect(app_handle: &tauri::AppHandle) -> Result<bool> {
        let store = app_handle.store(APP_STATE_FILENAME).unwrap();
        let shell = Shell::new()?;
        let shell_name = detect_shell()?;

        let uv_path = store
            .get("uv_path")
            .and_then(|s| s.as_str().map(String::from))
            .unwrap_or("".to_owned());

        if (!uv_path.is_empty()) {
            if let Ok(metadata) = fs::metadata(&uv_path) {
                if metadata.is_dir() || metadata.is_symlink() {
                    debug!("UV path exists: {}", uv_path);
                    return Ok(true);
                }
            }
            debug!("UV path does not exist: {}", uv_path);
        }

        debug!("Running check node command");

        #[cfg(target_os = "macos")]
        let cmd_output = cmd!(shell, "{shell_name} -ic 'which uv'").read()?;

        #[cfg(target_os = "windows")]
        let cmd_output = cmd!(shell, "where.exe uv").quiet().read()?;
        debug!("uv command output: {}", cmd_output);

        store.set("uv_path", cmd_output);
        store.set("use_system_uv", true);

        Ok(true)
    }

    pub async fn install(app_handle: &tauri::AppHandle) -> Result<()> {
        trace!("Installing UV");
        let store = app_handle.store(APP_STATE_FILENAME)?;
        let home_dir_str = get_home()?.to_string_lossy().to_string();

        #[cfg(target_os = "macos")]
        let (uv_version, uv_arch, uv_dir_name, uv_download_url) = {
            let uv_version = "0.5.5";
            let uv_arch = if cfg!(target_arch = "aarch64") {
                "aarch64-apple-darwin"
            } else {
                "x86_64_apple-darwin"
            };
            let uv_dir_name = format!("uv-{}-{}", uv_version, uv_arch);
            let uv_download_url = format!(
                "https://github.com/astral-sh/uv/releases/download/{}/uv-{}.tar.gz",
                uv_version, uv_arch
            );
            (uv_version, uv_arch, uv_dir_name, uv_download_url)
        };

        #[cfg(target_os = "windows")]
        let (uv_version, uv_arch, uv_dir_name, uv_download_url) = {
            let uv_version = "0.5.5";
            let uv_arch = if cfg!(target_arch = "x86_64") {
                "x86_64"
            } else {
                "i686"
            };
            let uv_dir_name = format!("uv-{}-{}", uv_version, uv_arch);
            let uv_download_url = format!(
                "https://github.com/astral-sh/uv/releases/download/{}/uv-{}-pc-windows-msvc.zip",
                uv_version, uv_arch
            );
            (uv_version, uv_arch, uv_dir_name, uv_download_url)
        };

        trace!("Downloading uv from {}", uv_download_url);

        // Create .node directory using fs
        #[cfg(target_os = "macos")]
        let uv_dir = format!("{}/.uv/{}", home_dir_str, uv_dir_name);
        #[cfg(target_os = "windows")]
        let uv_dir = format!("{}\\AppData\\Local\\uv", home_dir_str);

        trace!("Creating uv directory at {}", uv_dir);
        fs::create_dir_all(&uv_dir)?;

        // Download using reqwest async
        trace!("Downloading uv");
        let response = reqwest::get(uv_download_url).await?;
        let bytes = response.bytes().await?;

        // Extract archive
        trace!("Extracting archive");
        #[cfg(target_os = "macos")]
        {
            let gz = GzDecoder::new(Cursor::new(bytes));
            let mut archive = Archive::new(gz);
            archive.unpack(&uv_dir)?;
        }

        #[cfg(target_os = "windows")]
        {
            let cursor = Cursor::new(bytes);
            let mut archive = ZipArchive::new(cursor)?;
            archive.extract(&uv_dir)?;
            debug!("Extracted archive to {}", uv_dir);
        }

        store.set("uv_path", uv_dir);
        store.set("use_system_uv", false);
        trace!("All done");
        Ok(())
    }
}

impl ResourceHandler {
    async fn download(app_handle: &tauri::AppHandle) -> Result<()> {
        let store = app_handle.store(APP_STATE_FILENAME)?;
        let servers_json = reqwest::get(SERVERS_URL).await?.text().await?;
        debug!("servers.json: {}", servers_json);
        store.set("servers", servers_json);
        debug!("servers.json set in store");
        Ok(())
    }
    pub async fn detect(app_handle: &tauri::AppHandle) -> Result<bool> {
        let store = app_handle.store(APP_STATE_FILENAME)?;
        if store.get("servers").is_some() {
            return Ok(true);
        }
        Self::download(app_handle).await?;
        Ok(true)
    }
}
