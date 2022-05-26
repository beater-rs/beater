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
    #[serde(rename = "words")]
    pub text: Vec<InnerLyricsLine>,
    pub time: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct InnerLyricsLine {
    #[serde(rename = "string")]
    pub text: String,
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

    pub async fn into_lrc_file(self) -> crate::Result<String> {
        let mut lines = Vec::new();
        for line in self.lyrics.lines {
            for word in line.text {
                lines.push(format!("[{}]{}", line.time, word.text));
            }
        }
        Ok(lines.join("\n"))
    }
}
