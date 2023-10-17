use super::{email_address::EmailAddress, CreateTable, Table};
use serde::{Deserialize, Serialize};
use surrealdb::opt::RecordId;

const TABLE_NAME: &str = "account";

/// Represents the JSON object that a user would submit
/// to create an account
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAccountDto {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAccountDb {
    email: String,
    password: String,
    id: RecordId,
}

impl CreateTable for CreateAccountDb {
    fn id(&self) -> &str {
        &self.email
    }
}

impl From<CreateAccountDto> for CreateAccountDb {
    fn from(value: CreateAccountDto) -> Self {
        Self {
            email: value.email.clone(),
            password: value.password,
            id: RecordId::from((Account::name(), value.email.as_str())),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Account {
    pub id: Option<RecordId>,
    pub email: Option<EmailAddress>,
    pub name: Option<String>,
    pub confirmed: Option<bool>,
    pub instances: Option<Vec<RecordId>>,
}

impl Table for Account {
    fn name() -> &'static str {
        TABLE_NAME
    }
}
