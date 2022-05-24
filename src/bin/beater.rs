use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{error, fs, io, path::PathBuf};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    username: Option<String>,
    #[clap(short, long)]
    password: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let credentials_file = std::env::var("BEATER_CREDENTIALS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::config_dir()
                .unwrap()
                .join("beater")
                .join("credentials.toml")
        });

    let args = Args::parse();

    match fs::read_to_string(&credentials_file) {
        Ok(contents) => {
            let credentials = toml::from_str::<Credentials>(&contents)?;
            println!("{:#?}", credentials);
        }
        Err(e) => {
            println!("{}", e);
        }
    };
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
struct Credentials {
    pub username: String,
    pub password: String,
}
