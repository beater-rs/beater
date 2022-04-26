use librespot::{
    audio::AudioFile,
    core::{
        audio_key::AudioKeyError,
        channel::ChannelError,
        config::SessionConfig,
        mercury::MercuryError,
        session::{Session, SessionError},
        spotify_id::SpotifyId,
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
    /// Creates a new [`Beater`] instance. This will connect to your spotify account.
    ///
    /// <p style="background:rgba(255,181,77,0.16);padding:0.75em;">
    /// <strong>Warning:</strong> There is a <i>very slight</i> chance that you will be banned from Spotify. Use at your own risk.
    /// </p>
    ///
    /// # Examples
    ///
    /// ```
    /// use beater::Beater;
    /// let beater = Beater::new(username, password).await?;
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if the session fails to connect (e.g. your credentials are invalid,
    /// or [`librespot`] cannot access the Spotify API).
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

    /// Gets the
    ///
    /// # Examples
    ///
    /// ```
    /// use beater::Beater;
    ///
    /// let beater = Beater::new(username, password).await?;
    /// let song = beater.get_song(song_id).await?;
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if creating an [`AudioItem`] fails.
    pub async fn get_song(&self, id: SpotifyId) -> Result<AudioItem> {
        AudioItem::get_audio_item(&self.session, id)
            .await
            .map_err(Error::from)
    }

    /// Gets a readable audio file for the given [`AudioItem`], with the given [`FileFormat`].
    ///
    /// # Examples
    ///
    /// ```
    /// use beater::{Beater, FileFormat};
    /// use std::fs::File;
    ///
    /// // create a beater and get a song
    /// let beater: Beater = Beater::new(username, password).await?;
    /// let audio_item: AudioItem = beater.get_song(song_id).await?;
    /// let audio_file: AudioFile = beater.get_audio_file(audio_item, FileFormat::OGG_VORBIS_320).await?;
    ///
    /// // put the song into the buffer
    /// let audio_file = Vec::new();
    /// audio_file.read_to_end(&mut audio_file);
    ///
    /// // write the song to a file
    /// let mut file = File::create("song.ogg")?;
    /// file.write_all(&audio_file)?;
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if opening the [`AudioFile`] fails, or if the file format does not exist for the song.
    pub async fn get_audio_file(
        &self,
        audio_item: AudioItem,
        music_format: FileFormat,
    ) -> Result<AudioFile> {
        if let Some(file_id) = audio_item.files.get(&music_format) {
            AudioFile::open(&self.session, *file_id, 40 * 1024, true)
                .await
                .map_err(Error::from)

            // self.decrypt_audio_file(audio_item.id, file_id, audio_file)
            //     .await
        } else {
            Err(Error::FileFormatNotFound(
                music_format,
                audio_item.files.into_keys().collect(),
            ))
        }
    }

    /*
    pub(crate) async fn decrypt_audio_file<T: Read>(
        &self,
        spotify_id: SpotifyId,
        file_id: FileId,
        audio_file: T,
    ) -> Result<AudioDecrypt<T>> {
        let key = self
            .session
            .audio_key()
            .request(spotify_id, file_id)
            .await?;
         Ok(AudioDecrypt::new(key, audio_file))
    }
    */
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
