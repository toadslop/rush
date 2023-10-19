use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use surrealdb::opt::auth::Root;

#[derive(Debug, Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub database_name: String,
    pub connection: ConnectionType,
}

impl DatabaseSettings {
    pub fn get_root_credentials(&self) -> Root {
        Root {
            username: &self.username,
            password: self.password.expose_secret(),
        }
    }
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
