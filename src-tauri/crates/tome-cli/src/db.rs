use anyhow::{Context, Result};
use directories::ProjectDirs;
use rusqlite::Connection;
use std::path::PathBuf;

/// Finds the path to the SQLite database used by the Tome GUI application.
fn database_path() -> Option<PathBuf> {
    ProjectDirs::from("co", "runebook", "Tome")
        .map(|proj_dirs| proj_dirs.data_dir().join("tome.db"))
}

use crate::models::{ClientOptions, Engine};

/// Establishes a connection to the Tome database.
pub fn connect() -> Result<Connection> {
    let path = database_path().context("Could not determine application data directory.")?;
    Connection::open(&path).with_context(|| format!("Failed to open database at {}", path.display()))
}

// Define a custom error type to satisfy rusqlite's trait bounds.
#[derive(Debug)]
struct OptionsParseError(String);

impl std::fmt::Display for OptionsParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for OptionsParseError {}


/// Fetches all configured engines from the database.
pub fn get_engines(conn: &Connection) -> Result<Vec<Engine>> {
    let mut stmt = conn.prepare("SELECT id, name, type, options FROM engines")?;
    let engine_iter = stmt.query_map([], |row| {
        let options_str: String = row.get(3)?;
        let options: ClientOptions = serde_json::from_str(&options_str).map_err(|e| {
            let engine_name = row.get::<_, String>(1).unwrap_or_else(|_| "unknown".to_string());
            rusqlite::Error::FromSqlConversionFailure(
                3,
                rusqlite::types::Type::Text,
                Box::new(OptionsParseError(format!(
                    "Failed to parse options JSON for engine '{}': {}",
                    engine_name, e
                ))),
            )
        })?;

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
