use std::collections::HashMap;

use librespot_core::{Session, SpotifyId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Lyrics {
    pub lyrics: InnerLyrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InnerLyrics {
    pub provider: String,
    pub kind: LyricsKind,
    pub track_id: String,
    pub lines: Vec<LyricsLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct LyricsLine {
    pub words: Vec<HashMap<String, String>>,
    pub time: u32,
}

// TODO: figure out which other lyrics kinds are possible
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LyricsKind {
    Line,
}

impl Lyrics {
    pub async fn get(session: &Session, track_id: SpotifyId) -> crate::Result<Self> {
        serde_json::from_str(&String::from_utf8_lossy(
            &session.spclient().get_lyrics(track_id).await?,
        ))
        .map_err(|e| Box::new(e) as _)
    }
}
