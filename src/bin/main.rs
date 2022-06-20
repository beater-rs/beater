use beater::Beater;
use clap::{arg, command};
use librespot_core::{spotify_id::SpotifyItemType, SpotifyId};
use librespot_metadata::{audio::AudioFileFormat, Album, Artist, Metadata, Playlist, Track};
use std::{
    error, fs,
    path::{Path, PathBuf},
};

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

    let credentials_path = &*config_dir.join("credentials.toml");

    let args = command!()
        .arg(
            arg!( -c --credentials <FILE> "Path to credentials file" )
                .required(false)
                .forbid_empty_values(true)
                .default_value(&*credentials_path.to_string_lossy()),
        )
        .arg(
            arg!( -u --username <USERNAME> "Spotify username" )
                .required(!credentials_path.exists())
                .forbid_empty_values(true)
                .requires("password"),
        )
        .arg(
            arg!( -p --password <PASSWORD> "Spotify password" )
                .required(!credentials_path.exists())
                .forbid_empty_values(true)
                .requires("username"),
        )
        .arg(
            arg!( <url> "The Spotify url you want to download" ).validator(|v| {
                if v.starts_with("spotify:")
                    || ((v.starts_with("http://") || v.starts_with("https://"))
                        && v.contains("open.spotify.com"))
                {
                    Ok(())
                } else {
                    Err(String::from("Invalid url/uri"))
                }
            }),
        )
        .arg(arg!( --debug "Enable debug output" ).required(false))
        .arg(
            arg!( --quality <QUALITY> "Quality of the output file in kbps [default: 320 if premium account, 160 otherwise]" )
                .required(false)
                .possible_value("320")
                .possible_value("160")
                .possible_value("96"),
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
        args.value_of("credentials").map(Path::new).unwrap(),
        args.value_of("username").map(String::from),
        args.value_of("password").map(String::from),
    );

    tracing::info!("Logging into Spotify");
    let beater = match Beater::new(username, password).await {
        Ok(beater) => beater,
        Err(err) => {
            tracing::error!("Failed to create beater: {err}");
            std::process::exit(1);
        }
    };

    let quality = args.value_of("quality").map(|q| match q {
        "320" => AudioFileFormat::OGG_VORBIS_320,
        "160" => AudioFileFormat::OGG_VORBIS_160,
        "96" => AudioFileFormat::OGG_VORBIS_96,
        _ => unreachable!(),
    });

    let id = beater.parse_uri(args.value_of("url").unwrap())?;

    match id.item_type {
        SpotifyItemType::Track => download_track(&beater, id, quality, None).await?,
        SpotifyItemType::Album => {
            let album = Album::get(beater.session(), id).await?;

            for track in album.tracks().iter().copied() {
                download_track(&beater, track, quality, Some(Path::new(&album.name))).await?;
            }
        }
        SpotifyItemType::Playlist => {
            let playlist = Playlist::get(beater.session(), id).await?;

            for track in playlist.tracks().iter().copied() {
                download_track(&beater, track, quality, Some(Path::new(&playlist.name()))).await?;
            }
        }
        item_type => {
            tracing::error!("Unsupported item type: {item_type:?}");
            std::process::exit(1);
        }
    }
    Ok(())
}

async fn download_track(
    beater: &Beater,
    track: SpotifyId,
    quality: Option<AudioFileFormat>,
    prefix: Option<&Path>,
) -> Result<()> {
    let track = Track::get(beater.session(), track).await?;

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

    let (audio_file, _file_id) = beater.get_audio_file(track.id, quality).await?;

    let file_name = if let Some(path) = prefix {
        if !path.exists() {
            fs::create_dir_all(path)?;
        }

        path.join(&format!("{artists} - {track_name}"))
    } else {
        PathBuf::from(&format!("{artists} - {track_name}"))
    };

    fs::write(file_name.with_extension("ogg"), audio_file)?;

    if track.has_lyrics {
        fs::write(
            file_name.with_extension("lrc"),
            beater.get_lyrics(track.id).await?.into_lrc_file(),
        )?;
    }

    Ok(())
}
