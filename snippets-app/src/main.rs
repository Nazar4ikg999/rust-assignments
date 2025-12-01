use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    fs,
    io::{self, Read},
    path::PathBuf,
};

#[derive(Debug, Serialize, Deserialize, Default)]
struct SnippetStore {
    snippets: HashMap<String, String>,
}

impl SnippetStore {
    fn load(path: &PathBuf) -> Self {
        if let Ok(content) = fs::read_to_string(path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            SnippetStore::default()
        }
    }

    fn save(&self, path: &PathBuf) -> io::Result<()> {
        let data = serde_json::to_string_pretty(self).unwrap();
        fs::write(path, data)
    }
}

fn storage_path() -> PathBuf {
    PathBuf::from("snippets.json")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);

    let action = args.next().unwrap_or_default();
    let name = args.next();

    match action.as_str() {
        "--name" => {
            let name = name.expect("snippet name is required after --name");
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;

            let path = storage_path();
            let mut store = SnippetStore::load(&path);
            store.snippets.insert(name.clone(), buffer);
            store.save(&path)?;

            println!("Snippet '{name}' saved.");
        }
        "--read" => {
            let name = name.expect("snippet name is required after --read");
            let path = storage_path();
            let store = SnippetStore::load(&path);

            if let Some(code) = store.snippets.get(&name) {
                println!("{code}");
            } else {
                eprintln!("Snippet '{name}' not found.");
            }
        }
        "--delete" => {
            let name = name.expect("snippet name is required after --delete");
            let path = storage_path();
            let mut store = SnippetStore::load(&path);

            if store.snippets.remove(&name).is_some() {
                store.save(&path)?;
                println!("Snippet '{name}' deleted.");
            } else {
                eprintln!("Snippet '{name}' not found.");
            }
        }
        _ => {
            eprintln!(
                "Usage:
  echo \"code\" | snippets-app --name \"Cool Rust pattern\"
  snippets-app --read \"Cool Rust pattern\"
  snippets-app --delete \"Cool Rust pattern\""
            );
        }
    }

    Ok(())
}