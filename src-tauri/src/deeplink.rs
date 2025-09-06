use tauri::{AppHandle, Emitter, Url};
use anyhow::Result;

pub fn mcp_install(query: &str, app_handle: &AppHandle) -> Result<()> {
    app_handle.emit("mcp/install", query)?;
    Ok(())
}

pub fn handle(urls: Vec<Url>, app_handle: &AppHandle) {
    if urls.is_empty() {
        log::warn!("User likely clicked an empty tome: link?");
        return;
    }

    let url = urls[0].clone();
    let path = url.path();

    match path {
        "mcp/install" => {
            if let Some(query) = url.query() {
                if let Err(e) = mcp_install(query, app_handle) {
                    log::error!("Failed to handle mcp/install deeplink: {}", e);
                }
            } else {
                log::error!("mcp/install deeplink called without a query string");
            }
        }
        _ => {
            log::warn!("Unknown runebook function for {:?}", path);
        }
    }
}
