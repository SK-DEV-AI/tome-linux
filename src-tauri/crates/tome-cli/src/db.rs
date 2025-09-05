use anyhow::{Context, Result};
use directories::ProjectDirs;
use rusqlite::Connection;
use std::path::PathBuf;

/// Finds the path to the SQLite database used by the Tome GUI application.
fn database_path() -> Result<PathBuf> {
    // The project identifier is `co.runebook` as defined in tauri.conf.json
    if let Some(proj_dirs) = ProjectDirs::from("co", "runebook", "Tome") {
        let data_dir = proj_dirs.data_dir();
        let db_path = data_dir.join("tome.db");
        if db_path.exists() {
            Ok(db_path)
        } else {
            anyhow::bail!("Database not found at {}", db_path.display())
        }
    } else {
        anyhow::bail!("Could not determine application data directory.")
    }
}

use crate::models::{ClientOptions, Engine};

/// Establishes a connection to the Tome database.
pub fn connect() -> Result<Connection> {
    let path = database_path()?;
    Connection::open(path).context("Failed to open database connection")
}

/// Fetches all configured engines from the database.
pub fn get_engines(conn: &Connection) -> Result<Vec<Engine>> {
    let mut stmt = conn.prepare("SELECT id, name, type, options FROM engines")?;
    let engine_iter = stmt.query_map([], |row| {
        let options_str: String = row.get(3)?;
        let options: ClientOptions =
            serde_json::from_str(&options_str).unwrap_or(ClientOptions {
                url: None,
                api_key: None,
            });

        Ok(Engine {
            id: row.get(0)?,
            name: row.get(1)?,
            r#type: row.get(2)?,
            options,
        })
    })?;

    let mut engines = Vec::new();
    for engine in engine_iter {
        engines.push(engine?);
    }
    Ok(engines)
}
