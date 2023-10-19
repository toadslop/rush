use config::{Config, ConfigError};
use serde::Deserialize;
use std::fmt::Display;

use self::{app::ApplicationSettings, db::DatabaseSettings, mail::MailSettings};

pub mod app;
pub mod db;
pub mod mail;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub mail: MailSettings,
}

#[tracing::instrument(name = "Loading configuration")]
pub fn get_configuration() -> Result<Settings, ConfigError> {
    tracing::debug!("Loading configuration");
    let base_path = std::env::current_dir()
        .map_err(|e| tracing::error!("Failed to locate current_dir: {e}"))
        .unwrap();
    tracing::trace!("Config base path is: {:?}", base_path);

    let configuration_directory = base_path.join("config");
    tracing::trace!("Config directory path is: {:?}", configuration_directory);

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "dev".into())
        .try_into()
        .map_err(|e| tracing::error!("Failed to load env var APP_ENVIRONEMTN: {e}"))
        .unwrap();
    tracing::trace!("App environment is: {:?}", environment);

    let environment_filename = format!("config.{}.yaml", environment.as_ref());
    tracing::trace!("Environment filename is: {:?}", environment_filename);

    let settings = Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("config.yaml"),
        ))
        .add_source(config::File::from(
            configuration_directory.join(environment_filename),
        ))
        .build()?;
    tracing::trace!("Settings loaded {:?}", settings);

    let settings = settings.try_deserialize::<Settings>();
    tracing::debug!("Configuration loaded");
    settings
}

#[derive(Debug)]
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
