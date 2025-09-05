pub(crate) mod process;
pub(crate) mod server;

use std::collections::HashMap;

use crate::state::{RunningSession, State};

use anyhow::{anyhow, Result};
use rmcp::model::CallToolRequestParam;
use rmcp::model::Tool;
use server::McpServer;
use tauri::{AppHandle, Manager};
use tokio::process::Command;

// This function is now Linux-only, so no platform-specific logic is needed.
pub fn get_os_specific_command(command: &str, app: &AppHandle) -> Result<Command> {
    let os_specific_command = match command {
        "python" => "python",
        "uvx" => "uvx",
        "node" => "node",
        "npx" => "npx",
        "bunx" => "bunx",
        _ => return Err(anyhow!("{} servers not supported.", command)),
    };

    app.path()
        .resolve(os_specific_command, tauri::path::BaseDirectory::Resource)
        .map(Command::new)
        .map_err(anyhow::Error::from)
}

pub async fn bootstrap(app: AppHandle) -> Result<()> {
    let mut uvx = get_os_specific_command("uvx", &app)?;
    uvx.arg("--help");
    uvx.kill_on_drop(true);

    let mut npx = get_os_specific_command("npx", &app)?;
    npx.arg("--help");
    npx.kill_on_drop(true);

    uvx.output().await?;
    npx.output().await?;

    Ok(())
}

pub async fn start(
    session_id: i32,
    command: String,
    args: Vec<String>,
    env: HashMap<String, String>,
    app: AppHandle,
) -> Result<()> {
    let handle = app.clone();
    let state = handle.state::<State>();
    let server = McpServer::start(command, args, env, app).await?;

    let mut sessions = state.sessions.lock().await;

    let mut session = sessions.remove(&session_id).unwrap_or_default();

    if session.mcp_servers.contains_key(&server.name()) {
        server.kill()?;
        sessions.insert(session_id, session);
        return Ok(());
    }

    server.tools().await?.iter().for_each(|tool| {
        session.tools.insert(tool.name.to_string(), server.name());
    });

    session.mcp_servers.insert(server.name(), server);
    sessions.insert(session_id, session);

    Ok(())
}

pub async fn stop(session_id: i32, name: String, state: tauri::State<'_, State>) -> Result<()> {
    let mut sessions = state.sessions.lock().await;

    if let Some(mut session) = sessions.remove(&session_id) {
        if let Some(server) = session.mcp_servers.remove(&name) {
            server.kill()?;
        }
        sessions.insert(session_id, session);
    }

    Ok(())
}

pub async fn stop_session(session_id: i32, state: tauri::State<'_, State>) -> Result<()> {
    let mut sessions = state.sessions.lock().await;

    if let Some(session) = sessions.remove(&session_id) {
        for server in session.mcp_servers.values() {
            server.kill()?;
        }
    }

    Ok(())
}

pub async fn get_tools(session_id: i32, state: tauri::State<'_, State>) -> Result<Vec<Tool>> {
    let mut tools: Vec<Tool> = vec![];
    let sessions = state.sessions.lock().await;

    let running_session = match sessions.get(&session_id) {
        Some(s) => s,
        None => return Ok(vec![]),
    };

    for server in running_session.mcp_servers.values() {
        tools.extend(server.tools().await?)
    }

    Ok(tools)
}

pub async fn call_tool(
    session_id: i32,
    name: String,
    arguments: serde_json::Map<String, serde_json::Value>,
    state: tauri::State<'_, State>,
) -> Result<String> {
    let sessions = state.sessions.lock().await;

    let running_session = match sessions.get(&session_id) {
        Some(s) => s,
        None => return Err(anyhow!("Session {} not found", session_id)),
    };

    let service_name = match running_session.tools.get(&name) {
        Some(s) => s.clone(),
        None => return Err(anyhow!("Tool {} not found in session {}", name, session_id)),
    };

    let server = match running_session.mcp_servers.get(&service_name) {
        Some(s) => s,
        None => {
            return Err(anyhow!(
                "MCP Server {} not found for tool {}",
                service_name,
                name
            ))
        }
    };

    let tool_call = CallToolRequestParam {
        name: std::borrow::Cow::from(name),
        arguments: Some(arguments),
    };

    server.call_tool(tool_call).await
}

pub async fn peer_info(
    command: String,
    args: Vec<String>,
    env: HashMap<String, String>,
    app: AppHandle,
) -> Result<String> {
    let server = McpServer::start(command, args, env, app).await?;
    let peer_info = server.peer_info();
    server.kill()?;
    Ok(serde_json::to_string(&peer_info)?)
}

pub async fn rename_server(
    session_id: i32,
    old_name: String,
    new_name: String,
    state: tauri::State<'_, State>,
) -> Result<()> {
    let mut sessions = state.sessions.lock().await;

    if let Some(session) = sessions.get_mut(&session_id) {
        if let Some(server) = session.mcp_servers.get_mut(&old_name) {
            server.set_name(new_name.clone());

            let tools_to_update: Vec<String> = session
                .tools
                .iter()
                .filter(|(_, server_name)| *server_name == &old_name)
                .map(|(tool_name, _)| tool_name.clone())
                .collect();

            for tool_name in tools_to_update {
                session.tools.insert(tool_name, new_name.clone());
            }

            if let Some(server) = session.mcp_servers.remove(&old_name) {
                session.mcp_servers.insert(new_name, server);
            }
        }
    }

    Ok(())
}
