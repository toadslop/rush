use secrecy::Secret;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct MailSettings {
    pub smtp_host: String,
    pub http_host: Option<String>,
    pub http_port: Option<u16>,
    pub smtp_port: Option<u16>,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<Secret<String>>,
}
