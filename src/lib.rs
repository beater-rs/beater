pub mod config;

use librespot::{
    core::{
        config::SessionConfig,
        keymaster::{get_token, Token},
        mercury::MercuryError,
        session::{Session, SessionError},
    },
    discovery::Credentials,
};

use thiserror::Error;

pub struct Beater {
    session: Session,
    token: Token,
    pub config: config::Config,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    SessionError(#[from] SessionError),
    #[error("An unknown error given from a Murcury request")]
    MurcuryError,
}

impl From<MercuryError> for Error {
    fn from(_: MercuryError) -> Self {
        Self::MurcuryError
    }
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

impl Beater {
    pub async fn new(
        username: impl Into<String>,
        password: impl Into<String>,
        config: config::Config,
    ) -> Result<Self> {
        let session = Session::connect(
            SessionConfig {
                user_agent: concat!(
                    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) ",
                    "AppleWebKit/537.36 (KHTML, like Gecko) ",
                    "Chrome/100.0.4896.127 Safari/537.36"
                )
                .to_owned(),
                device_id: "0".to_owned(),
                proxy: None,
                ap_port: None,
            },
            Credentials::with_password(username, password),
            None,
        )
        .await
        .unwrap();

        let token = get_token(
            &session,
            "b8682ddfdcd04c31a14ebd926a7e7f07",
            "user-read-email,playlist-read-private,user-library-read",
        )
        .await?;

        Ok(Self {
            session,
            token,
            config,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    async fn create() -> Beater {
        Beater::new(
            "31woy7dllvxal6lcroelpl5s2rhu",
            "idrcwhattoputasapassw0rd",
            config::Config {
                download_quality: config::DownloadQuality::High,
            },
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn new_beater() {
        create().await;
    }
}
