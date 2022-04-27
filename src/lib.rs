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

    pub async fn get_song(&self, id: SpotifyId) -> Result<AudioItem> {
        AudioItem::get_audio_item(&self.session, id)
            .await
            .map_err(Error::from)
    }

    pub async fn get_audio_file(
        &self,
        audio_item: &AudioItem,
        music_format: FileFormat,
    ) -> Result<(AudioFile, FileId)> {
        if let Some(file_id) = audio_item.files.get(&music_format) {
            AudioFile::open(&self.session, *file_id, 40 * 1024, true)
                .await
                .map(|audio_file| (audio_file, *file_id))
                .map_err(Error::from)

            // self.decrypt_audio_file(audio_item.id, file_id, audio_file)
            //     .await
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
    ) -> Result<AudioDecrypt<AudioFile>> {
        let key = self
            .session
            .audio_key()
            .request(spotify_id, file_id)
            .await?;
        Ok(AudioDecrypt::new(key, audio_file))
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
        cell::RefCell,
        io::Read,
        sync::{Arc, Mutex},
    };

    use librespot::playback::{
        audio_backend::{Sink, SinkBuilder, SinkResult},
        config::PlayerConfig,
        convert::Converter,
        decoder::AudioPacket,
        player::Player,
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

        let _song = beater.get_song(spotify_id).await.unwrap();

        let audio_file = Vec::new();

        #[derive(Clone)]
        struct Thing(Arc<Mutex<Vec<u8>>>);

        impl Sink for Thing {
            fn write(
                &mut self,
                packet: &AudioPacket,
                _converter: &mut Converter,
            ) -> SinkResult<()> {
                for i in packet.oggdata().unwrap() {
                    log::debug!("{:?}", i);
                    self.0.lock().unwrap().push(*i);
                }

                Ok(())
            }
        }

        let thing = Thing(Arc::new(Mutex::new(audio_file)));

        let thing_ = thing.clone();
        let (mut player, _) = Player::new(
            PlayerConfig::default(),
            beater.into_session(),
            None,
            move || Box::new(thing_),
        );

        player.load(spotify_id, true, 0);
        player.play();

        player.await_end_of_track().await;

        std::fs::write("test.ogg", &**thing.0.lock().unwrap()).unwrap();
    }
}
