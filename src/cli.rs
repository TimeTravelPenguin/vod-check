use clap::{Args, Parser, Subcommand};
use dotenvy::dotenv;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Path to the configuration file
    #[arg(short, long, value_parser = clap::value_parser!(PathBuf), default_value = "config.toml")]
    pub config: PathBuf,

    /// Set the log level
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn from_env_and_args() -> Self {
        dotenv().ok();
        Self::parse()
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run the TUI to configure preferences and options
    Tui,
    /// Run the application to download videos
    Run {
        #[command(flatten)]
        args: RunArgs,
    },
}

#[derive(Args, Debug, Clone)]
pub struct RunArgs {
    #[arg(short, long, env)]
    pub twitch_client_id: String,
    #[arg(short, long, env)]
    pub twitch_client_secret: String,
}
