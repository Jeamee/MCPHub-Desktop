use anyhow::{anyhow, Context, Result};
use glob;
use home;
use log::{debug, trace};
use tauri::utils::config;
use std::path::PathBuf;
use tempfile::NamedTempFile;
use xshell::cmd;

use super::utils::get_proxied_shell;

pub struct NpmHandler;

impl NpmHandler {
    fn get_home() -> Result<PathBuf> {
        let current_home = home::home_dir().context("Failed to get home directory");
        if let Ok(home_path) = &current_home {
            trace!("Home directory: {}", home_path.to_string_lossy());
        }
        current_home
    }

    fn detect_shell() -> Result<String> {
        let shell = std::env::var("SHELL").context("Failed to get SHELL environment variable")?;
        let shell_name = std::path::Path::new(&shell)
        .file_name()
        .and_then(|name| name.to_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("Invalid shell path"))?;
        trace!("Detected shell: {}", shell_name);
        Ok(shell_name)
    }

    pub fn detect() -> Result<bool> {
        let home = Self::get_home()?;

        #[cfg(target_os = "macos")]
        let paths = vec![
            // Homebrew paths
            PathBuf::from("/usr/local/bin/npm"),
            PathBuf::from("/opt/homebrew/bin/npm"),
            // nvm paths
            home.join(".nvm/versions/node/*/bin/npm"),
            // Global npm path
            PathBuf::from("/usr/bin/npm"),
            // Volta paths
            home.join(".volta/bin/npm"),
            // nodenv paths
            home.join(".nodenv/shims/npm"),
        ];

        #[cfg(target_os = "windows")]
        let paths = vec![
            // Program Files paths
            PathBuf::from("C:\\Program Files\\nodejs\\npm.cmd"),
            PathBuf::from("C:\\Program Files (x86)\\nodejs\\npm.cmd"),
            // AppData paths for nvm-windows
            home.join("AppData\\Roaming\\nvm\\*\\npm.cmd"),
            // Scoop paths
            home.join("scoop\\apps\\nodejs\\current\\npm.cmd"),
            // Chocolatey paths
            PathBuf::from("C:\\ProgramData\\chocolatey\\bin\\npm.cmd"),
            // User profile paths
            home.join("AppData\\Roaming\\npm\\npm.cmd"),
        ];

        let found = paths.iter().any(|path| {
            if path.to_string_lossy().contains('*') {
                let result = glob::glob(path.to_string_lossy().as_ref())
                    .map(|glob_paths| {
                        let found_paths: Vec<_> = glob_paths.filter_map(Result::ok).collect();
                        debug!(
                            "Glob search for {}: found paths: {:?}",
                            path.display(),
                            found_paths
                        );
                        !found_paths.is_empty()
                    })
                    .unwrap_or(false);
                if result {
                    return true;
                }
            } else {
                let exists = path.exists();
                debug!(
                    "Direct path check for {}: exists: {}",
                    path.display(),
                    exists
                );
                if exists {
                    return true;
                }
            }
            false
        });
        debug!("All path checks completed. Found any match: {}", found);
        Ok(found)
    }

    pub fn install() -> Result<()> {
        trace!("Installing NVM");
        let shell = get_proxied_shell()?;
        trace!("Downloading NVM install script");
        cmd!(shell, "curl -fsSL https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh -o /tmp/nvm-install.sh").run()?;
        trace!("Running NVM install script");
        cmd!(shell, "bash /tmp/nvm-install.sh").run()?;
        trace!("Cleaning up install script");
        cmd!(shell, "rm /tmp/nvm-install.sh").run()?;

        trace!("Configuring NVM for current shell");
        let nvm_source = r#"
            export NVM_DIR="$HOME/.nvm"
            [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"  # This loads nvm
            [ -s "$NVM_DIR/bash_completion" ] && \. "$NVM_DIR/bash_completion"  # This loads nvm bash_completion
            "#;

        let home_dir = Self::get_home()?;
        let system_shell = Self::detect_shell()?;
        trace!("System shell: {}", system_shell);
        let config_file = match system_shell.as_str() {
            "bash" => home_dir.join(".bash_profile"),
            "zsh" => home_dir.join(".zshrc"),
            "fish" => home_dir.join(".config/fish/config.fish"),
            _ => return Err(anyhow!("Unsupported shell type")),
        };
        let config_file_path = config_file.to_string_lossy().into_owned();
        trace!("Config file path: {}", config_file_path);

        cmd!(shell, "echo {nvm_source} >> {config_file_path}").run()?;

        trace!("Installing Node v22.11.0");
        cmd!(shell, "zsh -i -c '. ~/.nvm/nvm.sh && nvm install v22.11.0 && nvm alias default v22.11.0'").run()?;
        trace!("All done");
        Ok(())
    }
}
