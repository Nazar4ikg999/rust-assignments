use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    fs,
    io::{self, Read},
    path::PathBuf,
};

use anyhow::{bail, Context, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Snippet {
    name: String,
    code: String,
    created_at: String, 
}

trait SnippetStorage {
    fn save(&mut self, snippet: &Snippet) -> Result<()>;
    fn get(&self, name: &str) -> Result<Option<Snippet>>;
    fn delete(&mut self, name: &str) -> Result<()>;
}

//
// -------- JSON STORAGE -----------
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
// -------- SQLITE STORAGE ----------
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
// -------- ВИБІР СХОВИЩА ЧЕРЕЗ ENV -----
//

fn build_storage_from_env() -> Result<Box<dyn SnippetStorage>> {
    let env_value =
        env::var("SNIPPETS_APP_STORAGE").unwrap_or_else(|_| "JSON:snippets.json".to_string());

    let (kind, path) = env_value.split_once(':').ok_or_else(|| {
        anyhow::anyhow!(
            "SNIPPETS_APP_STORAGE must look like \
             JSON:/path/to/snippets.json or SQLITE:/path/to/snippets.sqlite"
        )
    })?;

    let path = PathBuf::from(path);

    match kind {
        "JSON" => Ok(Box::new(JsonStorage::new(path))),
        "SQLITE" => Ok(Box::new(
            SqliteStorage::new(path)
                .context("Failed to initialize SQLite storage")?,
        )),
        other => bail!("Unsupported storage type: {other}"),
    }
}

fn now_iso() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn print_usage() {
    eprintln!(
        "Usage:
  echo \"code\" | snippets-app --name \"Cool Rust pattern\"
  snippets-app --read \"Cool Rust pattern\"
  snippets-app --delete \"Cool Rust pattern\"

Env:
  SNIPPETS_APP_STORAGE=JSON:/path/to/snippets.json
  SNIPPETS_APP_STORAGE=SQLITE:/path/to/snippets.sqlite"
    );
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1);

    let action = match args.next() {
        Some(a) => a,
        None => {
            print_usage();
            return Ok(());
        }
    };

    let mut storage = build_storage_from_env().context("Failed to build storage from env")?;

    match action.as_str() {
        "--name" => {
            let name = match args.next() {
                Some(n) => n,
                None => {
                    eprintln!("Error: --name requires snippet name argument");
                    print_usage();
                    return Ok(());
                }
            };

            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .context("Failed to read snippet code from stdin")?;

            let snippet = Snippet {
                name: name.clone(),
                code: buffer,
                created_at: now_iso(),
            };

            storage
                .save(&snippet)
                .with_context(|| format!("Failed to save snippet '{name}'"))?;
            println!("Snippet '{name}' saved.");
        }
        "--read" => {
            let name = match args.next() {
                Some(n) => n,
                None => {
                    eprintln!("Error: --read requires snippet name argument");
                    print_usage();
                    return Ok(());
                }
            };

            match storage
                .get(&name)
                .with_context(|| format!("Failed to read snippet '{name}'"))?
            {
                Some(snippet) => println!("{}", snippet.code),
                None => eprintln!("Snippet '{name}' not found."),
            }
        }
        "--delete" => {
            let name = match args.next() {
                Some(n) => n,
                None => {
                    eprintln!("Error: --delete requires snippet name argument");
                    print_usage();
                    return Ok(());
                }
            };

            storage
                .delete(&name)
                .with_context(|| format!("Failed to delete snippet '{name}'"))?;
            println!("Snippet '{name}' deleted (if it existed).");
        }
        _ => {
            eprintln!("Unknown command: {action}");
            print_usage();
        }
    }

    Ok(())
}