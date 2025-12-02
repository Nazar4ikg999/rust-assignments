use clap::Parser;
use serde::{Deserialize, Serialize};
use config::{Config, ConfigError, Environment, File};

#[derive(Debug, Serialize, Deserialize)]
struct AppConfig {
    pub debug: bool,
    pub log_level: String,
    pub log_file: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            debug: false,
            log_level: "info".to_string(),
            log_file: "app.log".to_string(),
        }
    }
}

impl AppConfig {
    fn from_sources(conf_path: &str) -> Result<Self, ConfigError> {
        let defaults = AppConfig::default();

        let builder = Config::builder()
            .set_default("debug", defaults.debug)?
            .set_default("log_level", defaults.log_level.clone())?
            .set_default("log_file", defaults.log_file.clone())?
            .add_source(File::with_name(conf_path).required(false))
            .add_source(Environment::with_prefix("CONF").separator("_"));

        builder.build()?.try_deserialize()
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "config-app",
    about = "Assignment 5: configuration demo (defaults + TOML + ENV + CLI)"
)]
struct Cli {
    #[arg(short, long)]
    debug: bool,

    #[arg(short, long, env = "CONF_FILE", default_value = "config.toml")]
    conf: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let mut cfg = AppConfig::from_sources(&cli.conf)?;

    if cli.debug {
        cfg.debug = true;
    }

    println!("{:#?}", cfg);

    Ok(())
}
