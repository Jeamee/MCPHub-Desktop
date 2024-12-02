use anyhow::{anyhow, Context, Result};
#[cfg(target_os = "macos")]
use flate2::read::GzDecoder;
use home;
use log::{debug, error, trace};
use reqwest;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::io::{Cursor, Read};
use std::path::PathBuf;
#[cfg(target_os = "macos")]
use tar::Archive;
use xshell::{cmd, Shell};
#[cfg(target_os = "windows")]
use zip::ZipArchive;

pub struct NpmHandler;
pub struct UVHandler;

impl NpmHandler {
    fn get_home() -> Result<PathBuf> {
        let current_home = home::home_dir().context("Failed to get home directory");
        if let Ok(home_path) = &current_home {
            trace!("Home directory: {}", home_path.to_string_lossy());
        }
        current_home
    }

    fn detect_shell() -> Result<String> {
        #[cfg(target_os = "macos")]
        {
            let shell =
                std::env::var("SHELL").context("Failed to get SHELL environment variable")?;
            let shell_name = std::path::Path::new(&shell)
                .file_name()
                .and_then(|name| name.to_str())
                .map(|s| s.to_string())
                .ok_or_else(|| anyhow!("Invalid shell path"))?;
            trace!("Detected shell: {}", shell_name);
            Ok(shell_name)
        }

        #[cfg(target_os = "windows")]
        {
            Ok("powershell".to_string())
        }
    }

    pub fn detect() -> Result<bool> {
        let shell = Shell::new()?;
        let shell_name = Self::detect_shell()?;

        debug!("Running check node command");

        #[cfg(target_os = "macos")]
        let cmd_output = cmd!(shell, "{shell_name} -ic 'which node'").read()?;

        #[cfg(target_os = "windows")]
        let cmd_output = match cmd!(
            shell,
            "where.exe node"
        )
        .output()
        {
            Ok(output) => {
                if !output.stderr.is_empty() {
                    error!(
                        "Command stderr: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                String::from_utf8_lossy(&output.stdout).to_string()
            }
            Err(e) => {
                error!("Failed to execute 'Get-Command node': {}", e);
                return Ok(false);
            }
        };

        debug!("Node command output: {}", cmd_output);
        let found = cmd_output.contains("node");

        Ok(found)
    }

    pub async fn install() -> Result<()> {
        trace!("Installing Node.js");
        let shell = Shell::new()?;
        let home_dir_str = Self::get_home()?.to_string_lossy().to_string();

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

        trace!("Configuring node");
        let home_dir = Self::get_home()?;

        #[cfg(target_os = "macos")]
        {
            let system_shell = Self::detect_shell()?;
            trace!("System shell: {}", system_shell);

            let config_file = match system_shell.as_str() {
                "bash" => home_dir.join(".bash_profile"),
                "zsh" => home_dir.join(".zshrc"),
                "fish" => home_dir.join(".config/fish/config.fish"),
                _ => return Err(anyhow!("Unsupported shell type")),
            };

            trace!("Config file path: {}", config_file.display());

            // Create config file if it doesn't exist
            if !config_file.exists() {
                if let Some(parent) = config_file.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::File::create(&config_file)?;
            }

            // Check if PATH is already configured
            let env_set_str = format!("export PATH=\"$HOME/.node/{}/bin:$PATH\"", node_dir_name);
            let file = fs::File::open(&config_file)?;
            let reader = std::io::BufReader::new(file);
            let already_configured = reader
                .lines()
                .any(|line| line.map(|l| l.contains(&env_set_str)).unwrap_or(false));

            // Append PATH only if not already configured
            if !already_configured {
                trace!("Adding PATH to shell config");
                let mut file = OpenOptions::new().append(true).open(&config_file)?;

                writeln!(file, "\n# Added by MCPHub-Desktop")?;
                writeln!(file, "{}", env_set_str)?;
            } else {
                trace!("PATH already configured, skipping");
            }

            trace!("fix links");
            let bin_dir = format!("{home_dir_str}/.node/{node_dir_name}/bin");
            let npm_modules_dir =
                format!("{home_dir_str}/.node/{node_dir_name}/lib/node_modules/npm/bin");

            // Remove existing symlinks if they exist
            let npm_path = format!("{bin_dir}/npm");
            let npx_path = format!("{bin_dir}/npx");
            if fs::metadata(&npm_path).is_ok() {
                fs::remove_file(&npm_path)?;
            }
            if fs::metadata(&npx_path).is_ok() {
                fs::remove_file(&npx_path)?;
            }

            // Create new symlinks
            std::os::unix::fs::symlink(format!("{npm_modules_dir}/npm-cli.js"), &npm_path)?;
            std::os::unix::fs::symlink(format!("{npm_modules_dir}/npx-cli.js"), &npx_path)?;
        }

        #[cfg(target_os = "windows")]
        {
            // Update system PATH for Windows
            let node_bin_path = format!("{}\\{}\\", node_dir, node_dir_name);
            let powershell_command = format!(
                "[Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path', 'User') + ';{}', 'User')",
                node_bin_path.replace("\\", "\\\\")
            );

            cmd!(shell, "powershell -Command {powershell_command}").run()?;

            // Create npm.cmd and npx.cmd if they don't exist
            let bin_dir = format!("{}\\{}\\", node_dir, node_dir_name);
            let npm_modules_dir = format!("{}\\node_modules\\npm\\bin", bin_dir);

            let npm_cmd_content = format!(
                "@ECHO off\r\nNODE_EXE=\"{}node.exe\"\r\nNPM_CLI_JS=\"{}\\npm-cli.js\"\r\n\"%NODE_EXE%\" \"%NPM_CLI_JS%\" %*",
                bin_dir, npm_modules_dir
            );
            let npx_cmd_content = format!(
                "@ECHO off\r\nNODE_EXE=\"{}node.exe\"\r\nNPX_CLI_JS=\"{}\\npx-cli.js\"\r\n\"%NODE_EXE%\" \"%NPX_CLI_JS%\" %*",
                bin_dir, npm_modules_dir
            );

            fs::write(format!("{}\\npm.cmd", bin_dir), npm_cmd_content)?;
            fs::write(format!("{}\\npx.cmd", bin_dir), npx_cmd_content)?;
        }

        trace!("All done");
        Ok(())
    }
}


impl UVHandler {
    fn get_home() -> Result<PathBuf> {
        let current_home = home::home_dir().context("Failed to get home directory");
        if let Ok(home_path) = &current_home {
            trace!("Home directory: {}", home_path.to_string_lossy());
        }
        current_home
    }

    fn detect_shell() -> Result<String> {
        #[cfg(target_os = "macos")]
        {
            let shell =
                std::env::var("SHELL").context("Failed to get SHELL environment variable")?;
            let shell_name = std::path::Path::new(&shell)
                .file_name()
                .and_then(|name| name.to_str())
                .map(|s| s.to_string())
                .ok_or_else(|| anyhow!("Invalid shell path"))?;
            trace!("Detected shell: {}", shell_name);
            Ok(shell_name)
        }

        #[cfg(target_os = "windows")]
        {
            Ok("powershell".to_string())
        }
    }

    pub fn detect() -> Result<bool> {
        let shell = Shell::new()?;
        let shell_name = Self::detect_shell()?;

        debug!("Running check node command");

        #[cfg(target_os = "macos")]
        let cmd_output = cmd!(shell, "{shell_name} -ic 'which uv'").read()?;

        #[cfg(target_os = "windows")]
        let cmd_output = match cmd!(
            shell,
            "where.exe uv"
        )
        .output()
        {
            Ok(output) => {
                if !output.stderr.is_empty() {
                    error!(
                        "Command stderr: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                String::from_utf8_lossy(&output.stdout).to_string()
            }
            Err(e) => {
                error!("Failed to execute 'which.exe uv': {}", e);
                return Ok(false);
            }
        };

        debug!("uv command output: {}", cmd_output);
        let found = cmd_output.contains("uv");

        Ok(found)
    }

    pub async fn install() -> Result<()> {
        trace!("Installing UV");
        let home_dir_str = Self::get_home()?.to_string_lossy().to_string();

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

        trace!("Configuring uv");
        let home_dir = Self::get_home()?;

        #[cfg(target_os = "macos")]
        {
            let system_shell = Self::detect_shell()?;
            trace!("System shell: {}", system_shell);

            let config_file = match system_shell.as_str() {
                "bash" => home_dir.join(".bash_profile"),
                "zsh" => home_dir.join(".zshrc"),
                "fish" => home_dir.join(".config/fish/config.fish"),
                _ => return Err(anyhow!("Unsupported shell type")),
            };

            trace!("Config file path: {}", config_file.display());

            // Create config file if it doesn't exist
            if !config_file.exists() {
                if let Some(parent) = config_file.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::File::create(&config_file)?;
            }

            // Check if PATH is already configured
            let env_set_str = format!("export PATH=\"$HOME/.uv/{}/:$PATH\"", uv_dir_name);
            debug!("env_set_str: {}", env_set_str);
            let file = fs::File::open(&config_file)?;
            let reader = std::io::BufReader::new(file);
            let already_configured = reader
                .lines()
                .any(|line| line.map(|l| l.contains(&env_set_str)).unwrap_or(false));

            // Append PATH only if not already configured
            if !already_configured {
                trace!("Adding PATH to shell config");
                let mut file = OpenOptions::new().append(true).open(&config_file)?;

                writeln!(file, "\n# Added by MCPHub-Desktop")?;
                writeln!(file, "{}", env_set_str)?;
            } else {
                trace!("PATH already configured, skipping");
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Update system PATH for Windows
            let powershell_command = format!(
                "[Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path', 'User') + ';{}', 'User')",
                uv_dir.replace("\\", "\\\\")
            );
            let shell = Shell::new()?;
            cmd!(shell, "powershell -Command {powershell_command}").run()?;
        }
        trace!("All done");
        Ok(())
    }
}
