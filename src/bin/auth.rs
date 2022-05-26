use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl Credentials {
    pub fn new(path: &Path, username: Option<String>, password: Option<String>) -> Credentials {
        match fs::read_to_string(path) {
            Ok(credentials) => match toml::from_str(&credentials) {
                Ok(credentials) => credentials,
                Err(err) => {
                    tracing::warn!("Failed to parse credentials file: {err}");
                    std::process::exit(1);
                }
            },
            Err(err) => {
                if err.kind() == io::ErrorKind::NotFound {
                    if let (Some(username), Some(password)) = (username, password) {
                        tracing::warn!(
                            "No credentials file found. Creating one with provided credentials..."
                        );
                        let credentials = Credentials { username, password };
                        let raw_credentials = toml::to_string(&credentials).unwrap();
                        fs::write(path, raw_credentials).unwrap();
                        credentials
                    } else {
                        tracing::error!(
                            "No credentials file found. Please provide both username and password."
                        );
                        std::process::exit(1);
                    }
                } else {
                    tracing::error!("Failed to read credentials file: {err}");
                    std::process::exit(1);
                }
            }
        }
    }
}
