pub(crate) mod process;
pub(crate) mod server;

use std::collections::HashMap;

use crate::state::State;

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
    let server_name = server.name().to_string();

    let mut sessions = state.sessions.lock().await;
    let session = sessions.entry(session_id).or_default();

    if session.mcp_servers.contains_key(&server_name) {
        server.kill()?;
        return Err(anyhow!("A server with the name '{}' is already running in this session.", server_name));
    }

    let tools = server.tools().await?;
    for tool in tools {
        session.tools.insert(tool.name.to_string(), server_name.clone());
    }

    session.mcp_servers.insert(server_name, server);

    Ok(())
}

pub async fn stop(session_id: i32, name: String, state: tauri::State<'_, State>) -> Result<()> {
    let mut sessions = state.sessions.lock().await;

    let session = sessions.get_mut(&session_id).ok_or_else(|| anyhow!("Session {} not found", session_id))?;

    if let Some(server) = session.mcp_servers.remove(&name) {
        server.kill()?;
        Ok(())
    } else {
        Err(anyhow!("Server '{}' not found in session {}", name, session_id))
    }
}

pub async fn stop_session(session_id: i32, state: tauri::State<'_, State>) -> Result<()> {
    let mut sessions = state.sessions.lock().await;

    if let Some(session) = sessions.remove(&session_id) {
        for server in session.mcp_servers.values() {
            server.kill()?;
        }
    }
    // This function is designed to succeed even if the session doesn't exist.
    Ok(())
}

pub async fn get_tools(session_id: i32, state: tauri::State<'_, State>) -> Result<Vec<Tool>> {
    let sessions = state.sessions.lock().await;

    let running_session = match sessions.get(&session_id) {
        Some(s) => s,
        None => return Ok(vec![]),
    };

    let mut tools: Vec<Tool> = vec![];
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

    let running_session = sessions.get(&session_id)
        .ok_or_else(|| anyhow!("Session {} not found", session_id))?;

    let service_name = running_session.tools.get(&name)
        .ok_or_else(|| anyhow!("Tool '{}' not found in session {}", name, session_id))?
        .clone();

    let server = running_session.mcp_servers.get(&service_name)
        .ok_or_else(|| anyhow!("MCP Server '{}' not found for tool '{}'", service_name, name))?;

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

    let session = sessions.get_mut(&session_id)
        .ok_or_else(|| anyhow!("Session {} not found", session_id))?;

    if let Some(server) = session.mcp_servers.remove(&old_name) {
        for (_, server_name) in session.tools.iter_mut() {
            if *server_name == old_name {
                *server_name = new_name.clone();
            }
        }
        session.mcp_servers.insert(new_name, server);
        Ok(())
    } else {
        Err(anyhow!("Server '{}' not found in session {}", old_name, session_id))
    }
}
