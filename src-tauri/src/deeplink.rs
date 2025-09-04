use tauri::{Emitter, Url};

use crate::APP_HANDLE;

pub fn mcp_install(query: &str) {
    if let Some(app_handle) = APP_HANDLE.get() {
        if let Err(e) = app_handle.emit("mcp/install", query) {
            log::error!("Failed to emit mcp/install event: {}", e);
        }
    } else {
        log::error!("App handle not initialized, could not emit mcp/install event.");
    }
}

pub fn handle(urls: Vec<Url>) {
    if urls.is_empty() {
        log::warn!("User likely clicked an empty tome: link?");
        return;
    }

    let url = urls[0].clone();
    let path = url.path();

    match path {
        "mcp/install" => {
            if let Some(query) = url.query() {
                mcp_install(query);
            } else {
                log::error!("mcp/install deeplink received without a query string.");
            }
        }
        _ => {
            log::warn!("Unknown runebook function for {:?}", path);
        }
    }
}
