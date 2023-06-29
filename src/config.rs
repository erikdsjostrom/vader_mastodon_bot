use anyhow::Result;
use elefren::data::Data;
use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = "config.toml";

/// Mastodon config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub mastodon: Data,
    pub location: String,
}

impl Config {
    /// Create a new config
    pub fn new() -> Result<Self> {
        Ok(toml::from_str(&std::fs::read_to_string(CONFIG_FILE)?)?)
    }
}
