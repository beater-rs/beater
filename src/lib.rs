use std::io::Read;

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
    metadata::{AudioItem, FileFormat},
};

use thiserror::Error;

pub struct Beater {
    pub(crate) session: Session,
}

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
    #[error("An unknown error given when requesting an `AudioKey`")]
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
    /// Creates a new [`Beater`]
    ///
    /// This connects with your Spotify account, and creates a token that can read your
    /// liked songs, private playlists, and your email
    pub async fn new(username: impl Into<String>, password: impl Into<String>) -> Result<Self> {
        let session = Session::connect(
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
        .await?;

        Ok(Self { session })
    }

    pub async fn get_song(&self, id: SpotifyId) -> Result<AudioItem> {
        AudioItem::get_audio_item(&self.session, id)
            .await
            .map_err(|e| Error::from(e))
    }

    pub async fn get_audio_file(
        &self,
        audio_item: AudioItem,
        music_format: FileFormat,
    ) -> Result<AudioFile> {
        match audio_item
            .files
            .get(&music_format)
            .map(|file_id| file_id.clone())
        {
            Some(file_id) => {
                AudioFile::open(&self.session, file_id, 40 * 1024, true)
                    .await
                    .map_err(|e| Error::from(e))

                // self.decrypt_audio_file(audio_item.id, file_id, audio_file)
                //     .await
            }
            None => Err(Error::FileFormatNotFound(
                music_format,
                audio_item.files.into_keys().collect(),
            )),
        }
    }

    // pub(crate) async fn decrypt_audio_file<T: Read>(
    //     &self,
    //     spotify_id: SpotifyId,
    //     file_id: FileId,
    //     audio_file: T,
    // ) -> Result<AudioDecrypt<T>> {
    //     let key = self
    //         .session
    //         .audio_key()
    //         .request(spotify_id, file_id)
    //         .await?;

    //     Ok(AudioDecrypt::new(key, audio_file))
    // }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

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
        let song = beater
            .get_song(SpotifyId::from_base62("2QTDuJIGKUjR7E2Q6KupIh").unwrap())
            .await
            .unwrap();
        let mut file = beater
            .get_audio_file(song, FileFormat::OGG_VORBIS_320)
            .await
            .unwrap();

        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();

        std::fs::write("song.ogg", buf).unwrap();
    }
}
