use librespot_core::{Session, SpotifyId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct Lyrics {
    pub lyrics: InnerLyrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct InnerLyrics {
    pub provider: String,
    pub kind: LyricsKind,
    pub track_id: String,
    pub lines: Vec<LyricsLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct LyricsLine {
    #[serde(rename = "words")]
    pub text: Vec<LyricWord>,
    pub time: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct LyricWord {
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

    pub async fn into_lrc_file(self) -> String {
        fn calculate_time(ms: u32) -> String {
            let secs = ms / 1000;
            let mins = secs / 60;
            let secs = secs % 60;
            format!("{:02}:{:02}", mins, secs)
        }

        let mut lines = Vec::new();
        for line in self.lyrics.lines {
            for word in line.text {
                lines.push(format!("[{}]{}", calculate_time(line.time), word.text));
            }
        }
        lines.join("\n")
    }
}
