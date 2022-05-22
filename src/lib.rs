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
    pub session: Session,
    pub cache: HashMap<FileId, AudioDecrypt<Cursor<Vec<u8>>>>,
}

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

    pub async fn get_audio_file(
        &self,
        track: SpotifyId,
        music_format: AudioFileFormat,
    ) -> Result<(AudioDecrypt<Cursor<Vec<u8>>>, FileId)> {
        use futures::stream::StreamExt;

        if let Some(file_id) = AudioItem::get_file(&self.session, track)
            .await?
            .files
            .get(&music_format)
            .copied()
        {
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

            let mut decrypted = self
                .decrypt_audio_file(track, file_id, Cursor::new(raw_res))
                .await?;

            // Skip the encryption header
            decrypted.seek(SeekFrom::Start(0xA7))?;

            Ok((decrypted, file_id))
        } else {
            Err(Error::not_found(""))
        }
    }

    pub(crate) async fn decrypt_audio_file<T: Read>(
        &self,
        track: SpotifyId,
        file: FileId,
        audio: T,
    ) -> Result<AudioDecrypt<T>> {
        Ok(AudioDecrypt::new(
            Some(self.session.audio_key().request(track, file).await?),
            audio,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::{env, fs};

    use crate::*;

    async fn create() -> Beater {
        let _ = dotenvy::from_filename(".env.test");
        let _ =
            simplelog::SimpleLogger::init(log::LevelFilter::Debug, simplelog::Config::default());

        Beater::new(
            env::var("SPOTIFY_USERNAME").expect("SPOTIFY_USERNAME must be set"),
            env::var("SPOTIFY_PASSWORD").expect("SPOTIFY_PASSWORD must be set"),
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn music_file() {
        let beater = create().await;

        // Test Drive - From How To Train Your Dragon Music From The Motion Picture.
        let track = SpotifyId::from_uri("spotify:track:2QTDuJIGKUjR7E2Q6KupIh").unwrap();

        let (mut audio_file, _file_id) = beater
            .get_audio_file(track, AudioFileFormat::OGG_VORBIS_160)
            .await
            .unwrap();

        let mut buf = Vec::new();
        audio_file.read_to_end(&mut buf).unwrap();

        let working = fs::read("test.ogg").unwrap();

        assert_eq!(buf, working);
    }
}
