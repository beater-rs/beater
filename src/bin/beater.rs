use beater::Beater;
use librespot::metadata::audio::AudioFileFormat;
use librespot::{core::SpotifyId, metadata::audio::AudioItem};
use std::{fs, process};

#[tokio::main]
async fn main() {
    let mut args = std::env::args();

    let bin = args.next().unwrap();
    let (username, password, track_id) = match args.len() {
        3 => (
            args.next().unwrap(),
            args.next().unwrap(),
            args.next().unwrap(),
        ),
        _ => {
            println!("Usage: {bin} <username> <password> <track-id>");
            process::exit(1);
        }
    };

    let mut beater = match Beater::new(username, password).await {
        Ok(beater) => beater,
        Err(err) => {
            println!("Error: {err}\nThis is probably due to an invalid username/password",);
            process::exit(1);
        }
    };

    let track_id = match SpotifyId::from_uri(&format!("spotify:track:{}", track_id)) {
        Ok(track_id) => track_id,
        Err(err) => {
            println!("Error: {err}\nThis is probably due to an invalid track-id",);
            process::exit(1);
        }
    };

    let song_name = AudioItem::get_file(&beater.session, track_id)
        .await
        .unwrap()
        .name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == ' ')
        .map(|c| c.to_ascii_lowercase())
        .collect::<String>();

    let (audio_file, _) = match beater
        .get_audio_file(track_id, AudioFileFormat::OGG_VORBIS_160)
        .await
    {
        Ok(track) => track,
        Err(err) => {
            println!("Error: {err}\nThis might be due to an invalid track-id",);
            process::exit(1);
        }
    };

    fs::write(format!("{song_name}.ogg"), audio_file).unwrap();
}
