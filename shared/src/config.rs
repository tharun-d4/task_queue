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
    pub lease_recovery: u8,
    pub cleanup: u8,
}

#[derive(Debug, Deserialize)]
pub struct Worker {
    pub heartbeat: u8,
    pub lease_duration: u8,
}

#[derive(Debug, Deserialize)]
pub struct MailServer {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub database: Database,
    pub server: Server,
    pub worker: Worker,
    pub mail_server: MailServer,
}

pub fn load_config(path: &str) -> Result<AppConfig, ConfigError> {
    let config = Config::builder()
        .add_source(File::new(path, FileFormat::Yaml))
        .build()?
        .try_deserialize()?;

    Ok(config)
}
