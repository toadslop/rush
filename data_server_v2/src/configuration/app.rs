use serde::Deserialize;

use super::Environment;

#[derive(Debug, Deserialize)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
    pub environment: Environment,
}
