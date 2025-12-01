use anyhow::{Context, Result};
use chrono::{SecondsFormat, Utc};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    fs,
    io::{self, Read},
    path::PathBuf,
};

use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Snippet {
    name: String,
    code: String,
    created_at: String,
}

/// Абстракція сховища
trait SnippetStorage {
    fn save(&mut self, snippet: &Snippet) -> Result<()>;
    fn get(&self, name: &str) -> Result<Option<Snippet>>;
    fn delete(&mut self, name: &str) -> Result<()>;
}

//
// -------- JSON STORAGE -------------
//

struct JsonStorage {
    path: PathBuf,
}

impl JsonStorage {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn load_map(&self) -> Result<HashMap<String, Snippet>> {
        if !self.path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&self.path).with_context(|| {
            format!(
                "Failed to read JSON storage from {}",
                self.path.display()
            )
        })?;

        if content.trim().is_empty() {
            return Ok(HashMap::new());
        }

        let map: HashMap<String, Snippet> =
            serde_json::from_str(&content).context("Failed to parse JSON storage")?;

        Ok(map)
    }

    fn save_map(&self, map: &HashMap<String, Snippet>) -> Result<()> {
        let data =
            serde_json::to_string_pretty(map).context("Failed to serialize snippets to JSON")?;
        fs::write(&self.path, data).with_context(|| {
            format!(
                "Failed to write JSON storage to {}",
                self.path.display()
            )
        })?;
        Ok(())
    }
}

impl SnippetStorage for JsonStorage {
    fn save(&mut self, snippet: &Snippet) -> Result<()> {
        let mut map = self.load_map()?;
        map.insert(snippet.name.clone(), snippet.clone());
        self.save_map(&map)
    }

    fn get(&self, name: &str) -> Result<Option<Snippet>> {
        let map = self.load_map()?;
        Ok(map.get(name).cloned())
    }

    fn delete(&mut self, name: &str) -> Result<()> {
        let mut map = self.load_map()?;
        map.remove(name);
        self.save_map(&map)
    }
}

//
// -------- SQLITE STORAGE -----------------
//

struct SqliteStorage {
    conn: rusqlite::Connection,
}

impl SqliteStorage {
    fn new(path: PathBuf) -> Result<Self> {
        use rusqlite::Connection;

        let conn = Connection::open(&path).with_context(|| {
            format!(
                "Failed to open SQLite database at {}",
                path.display()
            )
        })?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS snippets(
                name TEXT PRIMARY KEY,
                code TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )
        .context("Failed to create snippets table")?;

        Ok(Self { conn })
    }
}

impl SnippetStorage for SqliteStorage {
    fn save(&mut self, snippet: &Snippet) -> Result<()> {
        use rusqlite::params;

        self.conn
            .execute(
                "INSERT INTO snippets (name, code, created_at)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(name) DO UPDATE SET
                     code = excluded.code,
                     created_at = excluded.created_at",
                params![snippet.name, snippet.code, snippet.created_at],
            )
            .context("Failed to insert or update snippet in SQLite")?;

        Ok(())
    }

    fn get(&self, name: &str) -> Result<Option<Snippet>> {
        use rusqlite::{params, OptionalExtension};

        let row = self
            .conn
            .query_row(
                "SELECT name, code, created_at FROM snippets WHERE name = ?1",
                params![name],
                |row| {
                    Ok(Snippet {
                        name: row.get(0)?,
                        code: row.get(1)?,
                        created_at: row.get(2)?,
                    })
                },
            )
            .optional()
            .context("Failed to query snippet from SQLite")?;

        Ok(row)
    }

    fn delete(&mut self, name: &str) -> Result<()> {
        use rusqlite::params;

        self.conn
            .execute("DELETE FROM snippets WHERE name = ?1", params![name])
            .context("Failed to delete snippet from SQLite")?;

        Ok(())
    }
}

//
// -------- ВИБІР СХОВИЩА З ENV -------------
//

fn build_storage_from_env() -> Result<Box<dyn SnippetStorage>> {
    let env_value =
        env::var("SNIPPETS_APP_STORAGE").unwrap_or_else(|_| "JSON:snippets.json".to_string());

    let (kind, path) = env_value.split_once(':').ok_or_else(|| {
        anyhow::anyhow!(
            "SNIPPETS_APP_STORAGE must look like \
             JSON:/path/snippets.json or SQLITE:/path/snippets.sqlite"
        )
    })?;

    let path = PathBuf::from(path);

    match kind {
        "JSON" => Ok(Box::new(JsonStorage::new(path))),
        "SQLITE" => Ok(Box::new(SqliteStorage::new(path)?)),
        other => anyhow::bail!("Unsupported storage type: {other}"),
    }
}

fn now_iso() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

//
// -------- CLI + LOGGING -----------
//

#[derive(Parser, Debug)]
#[command(name = "snippets-app")]
struct Cli {
    #[arg(long)]
    name: Option<String>,

    #[arg(long)]
    read: Option<String>,

    #[arg(long)]
    delete: Option<String>,

    #[arg(long)]
    download: Option<String>,
}

fn init_tracing() {

    let level = env::var("SNIPPETS_APP_LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    let filter = EnvFilter::try_new(level.clone()).unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();
}

fn read_code(cli: &Cli) -> Result<String> {
    if let Some(url) = &cli.download {
        info!("Downloading snippet body from URL: {url}");
        let body = reqwest::blocking::get(url)
            .with_context(|| format!("Failed to GET {url}"))?
            .text()
            .with_context(|| format!("Failed to read body from {url}"))?;
        Ok(body)
    } else {
        info!("Reading snippet body from stdin");
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .context("Failed to read from stdin")?;
        Ok(buf)
    }
}

fn print_usage() {
    eprintln!(
        "Usage:
  echo \"code\" | snippets-app --name \"Cool Rust pattern\"
  snippets-app --name \"Cool Rust pattern\" --download \"https://.../snippet.txt\"
  snippets-app --read \"Cool Rust pattern\"
  snippets-app --delete \"Cool Rust pattern\""
    );
}

fn main() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();
    debug!("Parsed CLI: {:?}", cli);

    let mut storage = build_storage_from_env().context("Failed to initialize storage")?;

    if let Some(name) = cli.name.clone() {
        let code = read_code(&cli)?;
        let snippet = Snippet {
            name: name.clone(),
            code,
            created_at: now_iso(),
        };

        info!("Saving snippet '{name}'");
        storage
            .save(&snippet)
            .with_context(|| format!("Failed to save snippet '{name}'"))?;
        println!("Snippet '{name}' saved.");
        return Ok(());
    }

    if let Some(name) = cli.read.clone() {
        info!("Reading snippet '{name}'");
        match storage
            .get(&name)
            .with_context(|| format!("Failed to read snippet '{name}'"))?
        {
            Some(snippet) => println!("{}", snippet.code),
            None => {
                error!("Snippet '{name}' not found");
                eprintln!("Snippet '{name}' not found.");
            }
        }
        return Ok(());
    }

    if let Some(name) = cli.delete.clone() {
        info!("Deleting snippet '{name}'");
        storage
            .delete(&name)
            .with_context(|| format!("Failed to delete snippet '{name}'"))?;
        println!("Snippet '{name}' deleted (if it existed).");
        return Ok(());
    }

    print_usage();
    Ok(())
}

//
// -------- TESTS ---------
//

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    type TestResult = anyhow::Result<()>;

    fn example_snippet(name: &str) -> Snippet {
        Snippet {
            name: name.to_string(),
            code: "println!(\"hi\");".to_string(),
            created_at: now_iso(),
        }
    }

    #[test]
    fn json_storage_save_get_delete() -> TestResult {
        let dir = tempdir()?;
        let path = dir.path().join("snippets.json");
        let mut storage = JsonStorage::new(path);

        let snippet = example_snippet("json_snip");

        storage.save(&snippet)?;
        let got = storage
            .get("json_snip")?
            .expect("snippet must exist");
        assert_eq!(got.name, snippet.name);
        assert_eq!(got.code, snippet.code);

        storage.delete("json_snip")?;
        assert!(storage.get("json_snip")?.is_none());

        Ok(())
    }

    #[test]
    fn sqlite_storage_save_get_delete() -> TestResult {
        let dir = tempdir()?;
        let path = dir.path().join("snippets.sqlite");
        let mut storage = SqliteStorage::new(path)?;

        let snippet = example_snippet("sql_snip");

        storage.save(&snippet)?;
        let got = storage
            .get("sql_snip")?
            .expect("snippet must exist");
        assert_eq!(got.name, snippet.name);
        assert_eq!(got.code, snippet.code);

        storage.delete("sql_snip")?;
        assert!(storage.get("sql_snip")?.is_none());

        Ok(())
    }

    #[test]
    fn build_storage_from_env_json() -> TestResult {
        let dir = tempdir()?;
        let path = dir.path().join("env_snippets.json");

        std::env::set_var(
            "SNIPPETS_APP_STORAGE",
            format!("JSON:{}", path.display()),
        );

        let mut storage = build_storage_from_env()?;
        let snippet = example_snippet("env_json");
        storage.save(&snippet)?;
        let got = storage.get("env_json")?.expect("snippet must exist");
        assert_eq!(got.name, "env_json");

        std::env::remove_var("SNIPPETS_APP_STORAGE");
        Ok(())
    }

    #[test]
    fn build_storage_from_env_sqlite() -> TestResult {
        let dir = tempdir()?;
        let path = dir.path().join("env_snippets.sqlite");

        std::env::set_var(
            "SNIPPETS_APP_STORAGE",
            format!("SQLITE:{}", path.display()),
        );

        let mut storage = build_storage_from_env()?;
        let snippet = example_snippet("env_sql");
        storage.save(&snippet)?;
        let got = storage.get("env_sql")?.expect("snippet must exist");
        assert_eq!(got.name, "env_sql");

        std::env::remove_var("SNIPPETS_APP_STORAGE");
        Ok(())
    }
}