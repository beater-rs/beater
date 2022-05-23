//! <p style="background:rgba(255,181,77,0.16);padding:0.75em;">
//! <strong>Warning:</strong> There is a <i>very slight</i> chance that you will be banned from Spotify. Use at your own risk.
//! </p>

use std::{
    collections::HashMap,
    io::{Cursor, Read, Seek, SeekFrom},
};

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

pub struct Beater {
    session: Session,
    cache: HashMap<FileId, Vec<u8>>,
}

pub const ENCRYPTED_HEADER_SIZE: u8 = 0xA7;

pub(crate) type Result<T> = std::result::Result<T, Error>;

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
        Ok(Self {
            session,
            cache: HashMap::new(),
        })
    }

    /// Creates a new [`Beater`] instance.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use beater::Beater;
    ///
    /// let beater = Beater::new_with_session(session).await?;
    /// ```
    pub async fn new_with_session(session: Session) -> Self {
        Self {
            session,
            cache: HashMap::new(),
        }
    }

    pub async fn get_audio_file(
        &mut self,
        track: SpotifyId,
        music_format: AudioFileFormat,
    ) -> Result<(Vec<u8>, FileId)> {
        use futures::stream::StreamExt;

        if let Some(file_id) = AudioItem::get_file(&self.session, track)
            .await?
            .files
            .get(&music_format)
            .copied()
        {
            if let Some(decrypted) = self.cache.get(&file_id) {
                return Ok((decrypted.clone(), file_id));
            }

            let cdn_url = CdnUrl::new(file_id).resolve_audio(&self.session).await?;

            let req = http::Request::builder()
                .method(&http::Method::GET)
                .uri(cdn_url.try_get_url()?)
                .body(hyper::Body::empty())?;

            let mut res = self.session.http_client().request(req).await?.into_body();

            let mut raw_res = Vec::new();

            while let Some(Ok(chunk)) = res.next().await {
                raw_res.extend(&chunk);
            }

            let mut encrypted = Cursor::new(raw_res);
            // Skip the encryption header
            encrypted.seek(SeekFrom::Start(ENCRYPTED_HEADER_SIZE as u64))?;

            let audio_key = self.session.audio_key().request(track, file_id).await?;
            let encrypted_size = encrypted.get_ref().len() as u32;

            let mut decrypted_ = AudioDecrypt::new(Some(audio_key), encrypted);
            let mut decrypted = Vec::with_capacity(encrypted_size as usize);

            decrypted_.read_to_end(&mut decrypted)?;
            drop(decrypted_);

            self.cache.insert(file_id, decrypted.clone());

            Ok((decrypted, file_id))
        } else {
            Err(Error::not_found(""))
        }
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
        let track = SpotifyId::from_uri("spotify:track:2QTDuJIGKUjR7E2Q6KupIh").unwrap();

        let (audio_file, _file_id) = beater
            .get_audio_file(track, AudioFileFormat::OGG_VORBIS_160)
            .await
            .unwrap();

        let working = fs::read("test.ogg").unwrap();

        assert_eq!(audio_file, working);
    }
}
