//! <p style="background:rgba(255,181,77,0.16);padding:0.75em;">
//! <strong>Warning:</strong> There is a <i>very slight</i> chance that you will be banned from Spotify. Use at your own risk.
//! </p>
//!
use librespot::{
    audio::{AudioDecrypt, AudioFile},
    core::{
        config::SessionConfig,
        error::Error,
        session::Session,
        spotify_id::{FileId, SpotifyId},
    },
    discovery::Credentials,
    metadata::audio::{AudioFileFormat, AudioItem},
};

pub struct Beater {
    pub(crate) session: Session,
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
    /// let beater = Beater::new(username, password);
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
        Ok(Self { session })
    }

    pub async fn get_track(&self, id: SpotifyId) -> Result<AudioItem> {
        AudioItem::get_file(&self.session, id).await
    }

    pub async fn get_audio_file(
        &self,
        audio_item: &AudioItem,
        music_format: AudioFileFormat,
    ) -> Result<(AudioDecrypt<AudioFile>, FileId)> {
        if let Some(file_id) = audio_item.files.get(&music_format).copied() {
            let encrypted = AudioFile::open(&self.session, file_id, 1024 * 1024).await?;
            encrypted.get_stream_loader_controller()?.set_stream_mode();

            let decrypted = self
                .decrypt_audio_file(audio_item.id, file_id, encrypted)
                .await?;

            Ok((decrypted, file_id))
        } else {
            Err(Error::not_found(""))
        }
    }

    pub(crate) async fn decrypt_audio_file(
        &self,
        spotify_id: SpotifyId,
        file_id: FileId,
        audio_file: AudioFile,
    ) -> Result<AudioDecrypt<AudioFile>> {
        let key = self
            .session
            .audio_key()
            .request(spotify_id, file_id)
            .await?;
        Ok(AudioDecrypt::new(Some(key), audio_file))
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn into_session(self) -> Session {
        self.session
    }
}

#[cfg(test)]
mod tests {
    use std::{
        env,
        fs::OpenOptions,
        io::{Read, Seek, SeekFrom, Write},
    };

    use crate::*;

    async fn create() -> Beater {
        simplelog::SimpleLogger::init(log::LevelFilter::Debug, simplelog::Config::default()).ok();

        Beater::new(
            env::var("SPOTIFY_USERNAME")
                .unwrap_or_else(|_| "31woy7dllvxal6lcroelpl5s2rhu".to_owned()),
            env::var("SPOTIFY_PASSWORD").unwrap_or_else(|_| "idrcwhattoputasapassw0rd".to_owned()),
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn new_beater() {
        create().await;
    }

    #[tokio::test]
    async fn get_audio_file() {
        let beater = create().await;

        let spotify_id = SpotifyId::from_base62("2QTDuJIGKUjR7E2Q6KupIh").unwrap();

        let song = beater.get_track(spotify_id).await.unwrap();

        let (mut audio_file, _file_id) = beater
            .get_audio_file(&song, AudioFileFormat::OGG_VORBIS_160)
            .await
            .unwrap();

        audio_file.seek(SeekFrom::Start(0xA7)).unwrap();

        let mut buf = Vec::new();
        audio_file.read_to_end(&mut buf).unwrap();

        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .truncate(true)
            .open("test.ogg")
            .unwrap();

        file.write_all(&buf).unwrap();

        assert!(!buf.is_empty(), "the song is empty");
    }
}
