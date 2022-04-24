pub struct Config {
    pub download_quality: DownloadQuality,
}

pub enum DownloadQuality {
    Normal,
    High,
    VeryHigh,
}
