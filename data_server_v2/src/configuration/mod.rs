use std::fmt::Display;

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
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("config");
    println!("{}", configuration_directory.to_str().unwrap());

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");

    let environment_filename = format!("config.{}.yaml", environment.as_ref());

    let settings = Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("config.yaml"),
        ))
        .add_source(config::File::from(
            configuration_directory.join(environment_filename),
        ))
        .build()?;

    settings.try_deserialize::<Settings>()
}

pub enum Environment {
    Dev,
    Test,
    Prod,
}

impl AsRef<str> for Environment {
    fn as_ref(&self) -> &str {
        match self {
            Environment::Dev => "dev",
            Environment::Test => "test",
            Environment::Prod => "prod",
        }
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Dev => write!(f, "dev"),
            Environment::Test => write!(f, "test"),
            Environment::Prod => write!(f, "prod"),
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "dev" => Ok(Environment::Dev),
            "test" => Ok(Environment::Test),
            "prod" => Ok(Environment::Prod),
            other => Err(format!(
                "{} is not a supported environment. \
                Use either `dev`, `test`, or `prod`.",
                other
            )),
        }
    }
}
