use std::{fs, path::Path};

use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct FSConfig {
    pub base_dir: String
}

#[derive(Deserialize, Debug)]
pub struct StoreConfig {
    pub fs: Option<FSConfig>,
}

#[derive(Deserialize, Debug)]
pub struct AuthConfig {
    pub secret: String,
    pub allow_signup: bool,
}

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub store: StoreConfig,
    pub auth: AuthConfig,
    pub db_uri: String,
}

impl AppConfig {
    pub fn load_from_file(path: &Path) -> Result<AppConfig> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }
}
