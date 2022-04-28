//! <p class="compile_fail">
//!  **Warning**: There is a <i>slight</i> chance that you will get banned by using this library.
//!               Use at your own risk.
//! </p>

use librespot::{
    audio::{AudioDecrypt, AudioFile},
    core::{
        audio_key::AudioKeyError,
        channel::ChannelError,
        config::SessionConfig,
        mercury::MercuryError,
        session::{Session, SessionError},
        spotify_id::{FileId, SpotifyId},
    },
    discovery::Credentials,
    metadata::AudioItem,
};

use thiserror::Error;

pub struct Beater {
    pub(crate) session: Session,
}

pub use librespot::metadata::FileFormat;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    SessionError(#[from] SessionError),
    #[error("An unknown error given from a Murcury request")]
    MurcuryError,
    #[error("An unknown error given from a Channel request")]
    ChannelError,
    #[error("An `Option` ({0}) was None")]
    NoneOption(&'static str),
    #[error(
        "A file with the {0:?} file format was not found, the available file formats are {1:?}"
    )]
    FileFormatNotFound(FileFormat, Vec<FileFormat>),
    #[error(
        "An unknown error given when requesting an `AudioKey`. {}",
        "This usually means that you are requesting a file quality that is too high for your account"
    )]
    AudioKeyError,
}

impl From<MercuryError> for Error {
    fn from(_: MercuryError) -> Self {
        Self::MurcuryError
    }
}
impl From<ChannelError> for Error {
    fn from(_: ChannelError) -> Self {
        Self::ChannelError
    }
}
impl From<AudioKeyError> for Error {
    fn from(_: AudioKeyError) -> Self {
        Self::AudioKeyError
    }
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

impl Beater {
    /// Creates a new [`Beater`] instance.
    ///
    /// <p class="compile_fail">
    ///  **Warning**: There is a <i>slight</i> chance that you will get banned by using this library.
    ///               Use at your own risk.
    /// </p>
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
        Ok(Self {
            session: Session::connect(
                SessionConfig {
                    user_agent: concat!(
                        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) ",
                        "AppleWebKit/537.36 (KHTML, like Gecko) ",
                        "Chrome/100.0.4896.127 Safari/537.36"
                    )
                    .to_owned(),
                    device_id: String::new(),
                    proxy: None,
                    ap_port: None,
                },
                Credentials::with_password(username, password),
                None,
            )
            .await?,
        })
    }

    pub async fn get_audio_item(&self, id: SpotifyId) -> Result<AudioItem> {
        AudioItem::get_audio_item(&self.session, id)
            .await
            .map_err(Error::from)
    }

    pub async fn get_audio_file(
        &self,
        audio_item: &AudioItem,
        music_format: FileFormat,
    ) -> Result<(AudioDecrypt<AudioFile>, FileId)> {
        if let Some(file_id) = audio_item.files.get(&music_format).copied() {
            let encrypted = AudioFile::open(&self.session, file_id, 1024 * 1024, true)
                .await
                .map_err(Error::from)?;
            encrypted.get_stream_loader_controller().set_stream_mode();

            let decrypted = self
                .decrypt_audio_file(audio_item.id, file_id, encrypted)
                .await
                .ok_or(Error::AudioKeyError)?;

            Ok((decrypted, file_id))
        } else {
            Err(Error::FileFormatNotFound(
                music_format,
                audio_item.files.keys().cloned().collect(),
            ))
        }
    }

    pub(crate) async fn decrypt_audio_file(
        &self,
        spotify_id: SpotifyId,
        file_id: FileId,
        audio_file: AudioFile,
    ) -> Option<AudioDecrypt<AudioFile>> {
        let key = self
            .session
            .audio_key()
            .request(spotify_id, file_id)
            .await
            .ok()?;
        Some(AudioDecrypt::new(key, audio_file))
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
        fs::OpenOptions,
        io::{Read, Seek, SeekFrom, Write},
    };

    use crate::*;

    async fn create() -> Beater {
        simplelog::SimpleLogger::init(log::LevelFilter::Debug, simplelog::Config::default()).ok();

        Beater::new("31woy7dllvxal6lcroelpl5s2rhu", "idrcwhattoputasapassw0rd")
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

        let song = beater.get_audio_item(spotify_id).await.unwrap();

        let (mut audio_file, _file_id) = beater
            .get_audio_file(&song, FileFormat::OGG_VORBIS_160)
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
