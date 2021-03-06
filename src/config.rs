use std::path::PathBuf;

use anyhow::Result;
use libirc::client::data::Config as IrcConnectionConfig;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct IrcOzingerConfig {
    pub authline: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IrcConfig {
    #[serde(flatten)]
    pub connection: IrcConnectionConfig,
    pub channel: String,
    #[serde(default)]
    pub ignores: Vec<String>,
    pub ozinger: Option<IrcOzingerConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiscordConfig {
    pub token: String,
    pub channel_id: u64,
    pub webhook_id: u64,
    pub webhook_token: String,
    #[serde(default)]
    pub ignores: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub irc: IrcConfig,
    pub discord: DiscordConfig,
}

impl Config {
    pub fn from_path(path: impl Into<PathBuf>) -> Result<Self> {
        let mut config = libconfig::Config::default();
        config
            .merge(libconfig::File::from(path.into()))?
            .merge(libconfig::Environment::with_prefix("APP"))?;
        config.try_into().map_err(Into::into)
    }
}
