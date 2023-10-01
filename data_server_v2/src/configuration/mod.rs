use config::{Config, ConfigError};
use serde::Deserialize;
use surrealdb::opt::auth::Root;

#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}
#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub database_name: String,
    pub connection: ConnectionType,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ConnectionSettings {
    pub port: u16,
    pub host: String,
}

impl ConnectionSettings {
    fn get_conn_string(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ConnectionType {
    InMemory,
    Local(ConnectionSettings),
    Remote(ConnectionSettings),
}

impl ConnectionType {
    pub fn get_conn_string(&self) -> String {
        match &self {
            Self::InMemory => "mem://".into(),
            Self::Local(settings) => format!("ws://{}", settings.get_conn_string()),
            Self::Remote(settings) => format!("wss://{}", settings.get_conn_string()),
        }
    }
}

impl DatabaseSettings {
    pub fn get_root_credentials(&self) -> Root {
        Root {
            username: &self.username,
            password: &self.password,
        }
    }
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let settings = Config::builder()
        .add_source(config::File::new(
            "configuration.yaml",
            config::FileFormat::Yaml,
        ))
        .build()?;

    settings.try_deserialize::<Settings>()
}
