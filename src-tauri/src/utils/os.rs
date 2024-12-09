use anyhow::{anyhow, Context, Result};
use home;
use log::trace;
use std::path::PathBuf;

pub fn get_home() -> Result<PathBuf> {
    let current_home = home::home_dir().context("Failed to get home directory");
    if let Ok(home_path) = &current_home {
        trace!("Home directory: {}", home_path.to_string_lossy());
    }
    current_home
}

pub fn detect_shell() -> Result<String> {
    #[cfg(target_os = "macos")]
    {
        let shell = std::env::var("SHELL").context("Failed to get SHELL environment variable")?;
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
