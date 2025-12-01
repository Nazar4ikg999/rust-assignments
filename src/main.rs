use anyhow::Result;
use clap::{ArgAction, Parser};
use config as cfg;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct AppConfig {
    debug: bool,
    host: String,
    port: u16,
}

#[derive(Parser, Debug)]
#[command(
    name = "task_3_9",
    version,
    about = "Prints its configuration to STDOUT"
)]
struct Cli {

    #[arg(short = 'd', long = "debug", action = ArgAction::SetTrue)]
    debug: bool,


    #[arg(
        short = 'c',
        long = "conf",
        env = "CONF_FILE",
        default_value = "config.toml"
    )]
    conf: String,
}

fn build_config(cli: &Cli) -> Result<AppConfig> {

    let builder = cfg::Config::builder()
        .set_default("debug", false)?
        .set_default("host", "127.0.0.1")?
        .set_default("port", 8080u16)?;

    let builder = builder.add_source(cfg::File::with_name(&cli.conf).required(false));


    let builder = builder.add_source(
        cfg::Environment::with_prefix("CONF")
            .separator("_")
            .try_parsing(true),
    );

    let settings = builder.build()?;

    let mut config: AppConfig = settings.try_deserialize()?;


    if cli.debug {
        config.debug = true;
    }

    Ok(config)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = build_config(&cli)?;

    let toml_str = toml::to_string_pretty(&config)?;
    println!("{toml_str}");

    Ok(())
}