use beater::Beater;
use clap::{arg, command};
use librespot_metadata::{audio::AudioFileFormat, Artist, Metadata, Track};
use std::{error, fs, path::PathBuf};

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

mod auth;
use auth::Credentials;

// to trigger a rebuild when the Cargo.toml changes
const _: &str = include_str!("../../Cargo.toml");

#[tokio::main]
async fn main() -> Result<()> {
    let config_dir = std::env::var("BEATER_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs::config_dir().unwrap().join("beater"));

    let credentials_path = config_dir.join("credentials.toml");

    let args = command!()
        .arg(
            arg!(
                -c --credentials <FILE> "Path to credentials file"
            )
            .required(false),
        )
        .arg(
            arg!(
                -u --username <USERNAME> "Spotify username"
            )
            .required(!credentials_path.exists()),
        )
        .arg(
            arg!(
                -p --password <PASSWORD> "Spotify password"
            )
            .required(!credentials_path.exists()),
        )
        .arg(arg!(
                <url> "The Spotify url you want to download"
        ))
        .arg(
            arg!(
                --debug "Enable debug output"
            )
            .required(false),
        )
        .get_matches();

    tracing_subscriber::fmt()
        .with_max_level(if args.is_present("debug") {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .without_time()
        .init();

    match fs::create_dir_all(&config_dir) {
        Ok(_) => {}
        Err(err) => {
            if err.kind() != std::io::ErrorKind::AlreadyExists {
                tracing::error!("Failed to create config directory: {err}");
                std::process::exit(1);
            }
        }
    };

    let Credentials { username, password } = Credentials::new(
        &args
            .value_of("credentials")
            .map(PathBuf::from)
            .unwrap_or(credentials_path),
        args.value_of("username").map(String::from),
        args.value_of("password").map(String::from),
    );

    let mut beater = match Beater::new(username, password).await {
        Ok(beater) => beater,
        Err(err) => {
            tracing::error!("Failed to create beater: {err}");
            std::process::exit(1);
        }
    };

    let track_id = beater.parse_uri(args.value_of("url").unwrap()).unwrap();
    let track = Track::get(beater.session(), track_id).await?;

    let track_name = track
        .name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == ' ')
        .collect::<String>();

    let artists = futures_util::future::join_all(
        track
            .artists
            .iter()
            .map(|id| async { Artist::get(beater.session(), *id).await.unwrap() }),
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
    fs::write(format!("{file_name}.ogg"), audio_file)?;

    if track.has_lyrics {
        fs::write(
            format!("{file_name}.lrc"),
            beater.get_lyrics(track_id).await?.into_lrc_file(),
        )?;
    }

    Ok(())
}
