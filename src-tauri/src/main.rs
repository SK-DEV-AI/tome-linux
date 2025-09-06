#![warn(unused_extern_crates)]

mod commands;
mod daemon;
mod deeplink;
mod http;
mod mcp;
mod migrations;
mod process;
mod state;
mod window;

use std::sync::OnceLock;

use anyhow::Result;
use process::Process;
use tauri::{AppHandle, Manager, RunEvent};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_window_state::{AppHandleExt, StateFlags, WindowExt};

use crate::migrations::migrations;
use crate::state::State;
use crate::window::configure_window;

// Globally available app handle
static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

fn setup_app(app: &mut tauri::App) -> Result<()> {
    if APP_HANDLE.set(app.handle().clone()).is_err() {
        log::error!("Failed to set APP_HANDLE as it was already set.");
    }

    let window = app
        .get_window("main")
        .ok_or_else(|| anyhow::anyhow!("Couldn't get main window. This is a critical error."))?;

    log_panics::init();

    app.manage(State {
        sessions: Default::default(),
        watchers: Default::default(),
    });

    if let Err(e) = configure_window(&window) {
        log::error!("Failed to configure window: {}", e);
    }

    if let Err(e) = window.restore_state(StateFlags::all()) {
        log::warn!("Failed to restore window state: {}", e);
    }

    let handle = app.handle().clone();
    app.deep_link()
        .on_open_url(move |event| deeplink::handle(event.urls(), &handle));

    let handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = mcp::bootstrap(handle).await {
            log::error!("Failed to bootstrap MCP: {}", e);
        }
    });

    Ok(())
}

fn main() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {}))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(
            tauri_plugin_sql::Builder::new()
                .add_migrations("sqlite:tome.db", migrations())
                .build(),
        )
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("log".to_string()),
                    },
                ))
                .level(log::LevelFilter::Debug)
                .build(),
        )
        .setup(|app| {
            if let Err(e) = setup_app(app) {
                log::error!("Fatal error during setup: {}", e);
                // We can't recover from a setup error, so we exit.
                std::process::exit(1);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            crate::http::fetch,
            // MCP
            commands::get_metadata,
            commands::get_mcp_tools,
            commands::call_mcp_tool,
            commands::start_mcp_server,
            commands::stop_mcp_server,
            commands::rename_mcp_server,
            // Sessions
            commands::stop_session,
            // Misc
            commands::restart,
            commands::watch,
            commands::unwatch_all,
        ])
        .build(tauri::generate_context!());

    match app {
        Ok(app) => {
            app.run(|app, event| {
                if let RunEvent::Exit = event {
                    match Process::current() {
                        Ok(p) => {
                            if let Err(e) = p.kill_tree() {
                                log::error!("Failed to kill child processes on exit: {}", e);
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to get current process, cannot kill children: {}", e);
                        }
                    }
                } else if let RunEvent::ExitRequested { .. } = event {
                     let _ = app.save_window_state(StateFlags::all());
                }
            });
        }
        Err(e) => {
            // Use a native dialog to show the error if the app fails to build.
            // This is a fallback for when the logger and Tauri UI are not available.
            native_dialog::MessageDialog::new()
                .set_title("Tome - Fatal Error")
                .set_text(&format!("Failed to build the Tauri application: {}", e))
                .set_type(native_dialog::MessageType::Error)
                .show_alert()
                .expect("Failed to show native error dialog.");
        }
    }
}
