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

pub fn load_config() -> Result<AppConfig, ConfigError> {
    let config = Config::builder()
        .add_source(File::new("./config.yaml", FileFormat::Yaml))
        .build()?;

    let config = config.try_deserialize()?;
    dbg!(&config);

    Ok(config)
}
