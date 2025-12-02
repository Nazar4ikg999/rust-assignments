use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    error::Error,
    fs,
    io::{self, Read},
    path::PathBuf,
};

type DynError = Box<dyn Error + Send + Sync>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Snippet {
    name: String,
    code: String,
    created_at: String, 
}

/// Абстракція сховища (JSON або SQLite)
trait SnippetStorage {
    fn save(&mut self, snippet: &Snippet) -> Result<(), DynError>;
    fn get(&self, name: &str) -> Result<Option<Snippet>, DynError>;
    fn delete(&mut self, name: &str) -> Result<(), DynError>;
}

//
// -------- JSON STORAGE ---------------
//

struct JsonStorage {
    path: PathBuf,
}

impl JsonStorage {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn load_map(&self) -> Result<HashMap<String, Snippet>, DynError> {
        if !self.path.exists() {
            return Ok(HashMap::new());
        }
        let content = fs::read_to_string(&self.path)?;
        if content.trim().is_empty() {
            return Ok(HashMap::new());
        }
        let map: HashMap<String, Snippet> = serde_json::from_str(&content)?;
        Ok(map)
    }

    fn save_map(&self, map: &HashMap<String, Snippet>) -> Result<(), DynError> {
        let data = serde_json::to_string_pretty(map)?;
        fs::write(&self.path, data)?;
        Ok(())
    }
}

impl SnippetStorage for JsonStorage {
    fn save(&mut self, snippet: &Snippet) -> Result<(), DynError> {
        let mut map = self.load_map()?;
        map.insert(snippet.name.clone(), snippet.clone());
        self.save_map(&map)
    }

    fn get(&self, name: &str) -> Result<Option<Snippet>, DynError> {
        let map = self.load_map()?;
        Ok(map.get(name).cloned())
    }

    fn delete(&mut self, name: &str) -> Result<(), DynError> {
        let mut map = self.load_map()?;
        map.remove(name);
        self.save_map(&map)
    }
}

//
// -------- SQLITE STORAGE ---------------
//

struct SqliteStorage {
    conn: rusqlite::Connection,
}

impl SqliteStorage {
    fn new(path: PathBuf) -> Result<Self, DynError> {
        use rusqlite::Connection;
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS snippets (
                name TEXT PRIMARY KEY,
                code TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;
        Ok(Self { conn })
    }
}

impl SnippetStorage for SqliteStorage {
    fn save(&mut self, snippet: &Snippet) -> Result<(), DynError> {
        use rusqlite::params;
        self.conn.execute(
            "INSERT INTO snippets (name, code, created_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(name) DO UPDATE SET
                 code = excluded.code,
                 created_at = excluded.created_at",
            params![snippet.name, snippet.code, snippet.created_at],
        )?;
        Ok(())
    }

    fn get(&self, name: &str) -> Result<Option<Snippet>, DynError> {
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
            .optional()?;
        Ok(row)
    }

    fn delete(&mut self, name: &str) -> Result<(), DynError> {
        use rusqlite::params;
        self.conn
            .execute("DELETE FROM snippets WHERE name = ?1", params![name])?;
        Ok(())
    }
}


fn build_storage_from_env() -> Result<Box<dyn SnippetStorage>, DynError> {
    let env_value =
        env::var("SNIPPETS_APP_STORAGE").unwrap_or_else(|_| "JSON:snippets.json".to_string());

    let (kind, path) = env_value
        .split_once(':')
        .ok_or("SNIPPETS_APP_STORAGE must look like JSON:/path/file.json or SQLITE:/path/file.sqlite")?;

    let path = PathBuf::from(path);

    match kind {
        "JSON" => Ok(Box::new(JsonStorage::new(path))),
        "SQLITE" => Ok(Box::new(SqliteStorage::new(path)?)),
        other => Err(format!("Unsupported storage type: {other}").into()),
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

fn main() -> Result<(), DynError> {
    let mut args = env::args().skip(1);

    let action = match args.next() {
        Some(a) => a,
        None => {
            print_usage();
            return Ok(());
        }
    };

    let mut storage = build_storage_from_env()?;

    match action.as_str() {
        "--name" => {
            let name = args
                .next()
                .ok_or("--name requires snippet name as argument")?;
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;

            let snippet = Snippet {
                name: name.clone(),
                code: buffer,
                created_at: now_iso(),
            };

            storage.save(&snippet)?;
            println!("Snippet '{name}' saved.");
        }
        "--read" => {
            let name = args
                .next()
                .ok_or("--read requires snippet name as argument")?;
            match storage.get(&name)? {
                Some(snippet) => {
                    println!("{}", snippet.code);
                }
                None => eprintln!("Snippet '{name}' not found."),
            }
        }
        "--delete" => {
            let name = args
                .next()
                .ok_or("--delete requires snippet name as argument")?;
            storage.delete(&name)?;
            println!("Snippet '{name}' deleted (if it existed).");
        }
        _ => {
            print_usage();
        }
    }

    Ok(())
}