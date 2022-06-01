//! <p style="background:rgba(255,181,77,0.16);padding:0.75em;">
//! <strong>Warning:</strong> There is a <i>very slight</i> chance that you will be banned from Spotify. Use at your own risk.
//! </p>

pub mod lyrics;

use std::{
    collections::HashMap,
    io::{Cursor, Read, Seek, SeekFrom},
    sync::RwLock,
};

mod librespot;
use librespot::{
    audio::AudioDecrypt,
    core::{
        cdn_url::CdnUrl,
        config::SessionConfig,
        error::Error,
        session::Session,
        spotify_id::{FileId, SpotifyId},
    },
    discovery::Credentials,
    metadata::audio::{AudioFileFormat, AudioItem},
};
use lyrics::Lyrics;
use once_cell::sync::Lazy;

#[derive(Clone)]
pub struct Beater(Session);

pub const ENCRYPTED_HEADER_SIZE: u8 = 0xA7;

pub(crate) type Result<T> = std::result::Result<T, Error>;

pub(crate) static CACHE: Lazy<RwLock<HashMap<FileId, Vec<u8>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

impl Beater {
    /// Creates a new [`Beater`] instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use beater::Beater;
    ///
    /// let beater = Beater::new("username", "password").await?;
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if the [`Session`] fails to connect (e.g. your credentials are wrong).
    pub async fn new(username: impl Into<String>, password: impl Into<String>) -> Result<Self> {
        let session = Session::new(SessionConfig::default(), None);
        session
            .connect(Credentials::with_password(username, password))
            .await?;
        Ok(Self(session))
    }

    /// Creates a new [`Beater`] instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use beater::Beater;
    ///
    /// let beater = Beater::new_with_session(session).await?;
    /// ```
    pub async fn new_with_session(session: Session) -> Self {
        Self(session)
    }

    pub async fn get_audio_file<T: Into<Option<AudioFileFormat>>>(
        &mut self,
        track: SpotifyId,
        music_format: T,
    ) -> Result<(Vec<u8>, FileId)> {
        use futures_util::stream::StreamExt;

        let music_format = music_format.into().unwrap_or_else(|| {
            if self.is_premium() {
                tracing::info!("User is premium, using 320 kbps");
                AudioFileFormat::OGG_VORBIS_320
            } else {
                tracing::info!("User is not premium, using 160 kbps");
                AudioFileFormat::OGG_VORBIS_160
            }
        });

        if let Some(file_id) = AudioItem::get_file(self.session(), track)
            .await?
            .files
            .get(&music_format)
            .copied()
        {
            tracing::debug!("Found FileId: {}", file_id);

            if let Some(decrypted) = CACHE.read().unwrap().get(&file_id) {
                tracing::info!("Using cached audio file");
                return Ok((decrypted.clone(), file_id));
            }

            let cdn_url = CdnUrl::new(file_id).resolve_audio(self.session()).await?;
            let cdn_url = cdn_url.try_get_url()?;

            tracing::debug!("Got CDN URL: {cdn_url}");

            let req = http::Request::builder()
                .method(&http::Method::GET)
                .uri(cdn_url)
                .body(hyper::Body::empty())?;

            tracing::info!("Requesting encrypted audio file");
            let mut res = self.session().http_client().request(req).await?.into_body();

            let mut raw_res = Vec::new();

            while let Some(Ok(chunk)) = res.next().await {
                raw_res.extend(&chunk);
            }

            let encrypted = Cursor::new(raw_res);

            tracing::info!("Requesting decryption key");
            let audio_key = self.session().audio_key().request(track, file_id).await?;
            let encrypted_size = encrypted.get_ref().len();

            tracing::info!("Decrypting audio file");
            let mut decrypted_ = AudioDecrypt::new(Some(audio_key), encrypted);
            // Skip the encryption header
            decrypted_.seek(SeekFrom::Start(ENCRYPTED_HEADER_SIZE as u64))?;

            let mut decrypted = Vec::with_capacity(encrypted_size);

            decrypted_.read_to_end(&mut decrypted)?;
            drop(decrypted_);

            CACHE.write().unwrap().insert(file_id, decrypted.clone());

            Ok((decrypted, file_id))
        } else {
            Err(Error::not_found(""))
        }
    }

    pub fn parse_uri(&self, uri: impl AsRef<str>) -> Result<SpotifyId> {
        use url::Url;

        let uri = uri.as_ref();
        if uri.starts_with("spotify:") {
            return SpotifyId::from_uri(uri);
        } else {
            let url = Url::parse(uri)?;
            if url.host_str() == Some("open.spotify.com") {
                let mut path = url
                    .path_segments()
                    .ok_or_else(|| Error::invalid_argument(""))?;

                if let (Some(type_), Some(id)) = (path.next(), path.next()) {
                    return SpotifyId::from_uri(&format!("spotify:{}:{}", type_, id));
                }
            }
        }

        Err(Error::invalid_argument(""))
    }

    pub async fn get_lyrics(&self, track: SpotifyId) -> Result<Lyrics> {
        Lyrics::get(self.session(), track).await
    }

    pub fn is_premium(&self) -> bool {
        self.session()
            .get_user_attribute("type")
            .map(|t| t == "premium")
            .unwrap_or(false)
    }

    #[inline]
    pub fn session(&self) -> &Session {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use std::{env, fs};

    use super::*;

    async fn create() -> Beater {
        let _ = dotenvy::from_filename(".env.test");

        Beater::new(
            env::var("SPOTIFY_USERNAME").expect("SPOTIFY_USERNAME must be set"),
            env::var("SPOTIFY_PASSWORD").expect("SPOTIFY_PASSWORD must be set"),
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn music_file() {
        let mut beater = create().await;

        // Test Drive - From How To Train Your Dragon Music From The Motion Picture.
        let track = beater
            .parse_uri("spotify:track:2QTDuJIGKUjR7E2Q6KupIh")
            .unwrap();

        let (audio_file, _file_id) = beater
            .get_audio_file(track, AudioFileFormat::OGG_VORBIS_160)
            .await
            .unwrap();

        let working = fs::read("test.ogg").unwrap();

        // not using `assert_eq` because we don't want to print the files if they're different
        assert!(audio_file == working);
    }

    #[tokio::test]
    async fn lyrics() {
        use librespot_core::error::ErrorKind;

        let beater = create().await;

        // a song without lyrics
        {
            // Test Drive - From How To Train Your Dragon Music From The Motion Picture.
            let track = beater
                .parse_uri("spotify:track:2QTDuJIGKUjR7E2Q6KupIh")
                .unwrap();

            assert!(matches!(
                beater.get_lyrics(track).await,
                Err(Error {
                    kind: ErrorKind::NotFound,
                    ..
                })
            ));
        }

        // a song with lyrics
        {
            // Thirsty - AJR
            let track = beater
                .parse_uri("spotify:track:0iQJLCJyv6TTPUP4u4y8DJ")
                .unwrap();

            let working = std::fs::read_to_string("test.lrc").unwrap();

            assert_eq!(
                beater.get_lyrics(track).await.unwrap().into_lrc_file(),
                working
            );
        }
    }
}
