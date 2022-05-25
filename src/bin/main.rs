use auth::Credentials;
use beater::Beater;
use clap::Parser;
use librespot_core::SpotifyId;
use librespot_metadata::audio::AudioFileFormat;
use std::{error, fs, path::PathBuf};

mod auth;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    username: Option<String>,
    #[clap(short, long)]
    password: Option<String>,
    #[clap(short, long)]
    track_id: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .init();

    let config_dir = std::env::var("BEATER_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs::config_dir().unwrap().join("beater"));

    match fs::create_dir_all(&config_dir) {
        Ok(_) => {}
        Err(err) => {
            if err.kind() != std::io::ErrorKind::AlreadyExists {
                tracing::error!("Failed to create config directory: {err}");
                std::process::exit(1);
            }
        }
    };

    let credentials_path = config_dir.join("credentials.toml");

    let args = Args::parse();

    let Credentials { username, password } =
        auth::get_credentials(&credentials_path, args.username, args.password);

    let mut beater = match Beater::new(username, password).await {
        Ok(beater) => beater,
        Err(err) => {
            tracing::error!("Failed to create beater: {err}");
            std::process::exit(1);
        }
    };

    Ok(())
}
