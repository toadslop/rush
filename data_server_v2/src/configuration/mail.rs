use secrecy::Secret;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MailSettings {
    pub relay: String,
    pub smtp_username: String,
    pub smtp_password: Secret<String>,
}
