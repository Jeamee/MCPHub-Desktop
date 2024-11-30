use std::env;
use anyhow::Result;
use log::trace;
use xshell::Shell;

pub fn get_proxied_shell() -> Result<Shell> {
    let shell = Shell::new()?;

    // Get proxy settings from environment
    let proxies = [
        ("HTTP_PROXY", env::var("HTTP_PROXY").unwrap_or_default()),
        ("HTTPS_PROXY", env::var("HTTPS_PROXY").unwrap_or_default()),
        ("ALL_PROXY", env::var("ALL_PROXY").unwrap_or_default()),
        // Also set lowercase variants as some applications use them
        ("http_proxy", env::var("http_proxy").unwrap_or_default()),
        ("https_proxy", env::var("https_proxy").unwrap_or_default()),
        ("all_proxy", env::var("all_proxy").unwrap_or_default()),
    ];

    trace!("Proxies: {:#?}", proxies);
    // Set non-empty proxy variables
    for (key, value) in proxies.iter() {
        if !value.is_empty() {
            trace!("Setting {} = {}", key, value);
            shell.push_env(key, value);
        }
    }

    Ok(shell)
}