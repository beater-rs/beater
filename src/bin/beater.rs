use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[clap(short, long)]
    username: Option<String>,
    #[clap(short, long)]
    password: Option<String>,
}

#[tokio::main]
async fn main() {
    let credentials_file = std::env::var("BEATER_CREDENTIALS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::config_dir()
                .unwrap()
                .join("beater")
                .join("credentials.toml")
        });

    let args = Args::parse();
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
struct Credentials {
    pub username: String,
    pub password: String,
}
