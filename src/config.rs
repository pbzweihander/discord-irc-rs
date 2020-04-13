use {failure::Fallible, serde::Deserialize, std::path::PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct IrcOzingerConfig {
    pub authline: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IrcConfig {
    pub url: String,
    pub username: String,
    pub realname: String,
    pub nickname: String,
    pub channel: String,
    #[serde(default)]
    pub ignores: Vec<String>,
    pub ozinger: Option<IrcOzingerConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiscordConfig {
    pub token: String,
    pub channel_id: u64,
    pub webhook_url: String,
    #[serde(default)]
    pub ignores: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub irc: IrcConfig,
    pub discord: DiscordConfig,
}

impl Config {
    pub fn from_path(path: impl Into<PathBuf>) -> Fallible<Self> {
        let mut config = configc::Config::default();
        config
            .merge(configc::File::from(path.into()))?
            .merge(configc::Environment::with_prefix("APP"))?;
        config.try_into().map_err(Into::into)
    }
}
