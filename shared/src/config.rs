use config::{Config, ConfigError, File, FileFormat};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Database {
    pub url: String,
    pub pool_size: u8,
}

#[derive(Debug, Deserialize)]
pub struct Server {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub database: Database,
    pub server: Server,
}

pub fn load_config(path: &str) -> Result<AppConfig, ConfigError> {
    let config = Config::builder()
        .add_source(File::new(path, FileFormat::Yaml))
        .build()?
        .try_deserialize()?;

    Ok(config)
}
