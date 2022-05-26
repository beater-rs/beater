use beater::Beater;
use clap::Parser;
use librespot_core::SpotifyId;
use librespot_metadata::{audio::AudioFileFormat, Artist, Metadata, Track};
use std::{error, fs, path::PathBuf};

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

mod auth;
use auth::Credentials;

mod lyrics;
use lyrics::Lyrics;

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
async fn main() -> Result<()> {
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
        Credentials::new(&credentials_path, args.username, args.password);

    let mut beater = match Beater::new(username, password).await {
        Ok(beater) => beater,
        Err(err) => {
            tracing::error!("Failed to create beater: {err}");
            std::process::exit(1);
        }
    };

    let track_id = SpotifyId::from_uri(&format!("spotify:track:{}", args.track_id)).unwrap();
    let track = Track::get(&beater.session, track_id).await?;

    let track_name = track
        .name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == ' ')
        .collect::<String>();

    let artists = futures_util::future::join_all(
        track
            .artists
            .iter()
            .map(|id| async { Artist::get(&beater.session, *id).await.unwrap() }),
    )
    .await
    .into_iter()
    .map(|artist| {
        artist
            .name
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == ' ')
            .collect::<String>()
    })
    .collect::<Vec<String>>()
    .join(", ");

    let (audio_file, _file_id) = beater
        .get_audio_file(track_id, AudioFileFormat::OGG_VORBIS_160)
        .await?;

    let file_name = format!("{track_name} - {artists}");

    if track.has_lyrics {
        println!("{:#?}", Lyrics::get(&beater.session, track_id).await);
    }

    fs::write(format!("{file_name}.ogg"), audio_file)?;

    Ok(())
}
