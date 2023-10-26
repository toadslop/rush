use serde::Deserialize;

use super::Environment;

#[derive(Debug, Deserialize, Clone)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
    pub scheme: String,
    pub environment: Environment,
    pub domain: String,
}
