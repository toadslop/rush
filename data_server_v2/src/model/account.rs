use super::{email_address::EmailAddress, CreateTable, Table};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use surrealdb::opt::RecordId;
use uuid::Uuid;

const TABLE_NAME: &str = "account";
/// Represents the JSON object that a user would submit
/// to create an account
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreateAccountDto {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAccountDb {
    pub email: String,
    pub password: String,
    pub id: RecordId,
    pub name: String,
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
            name: value.name,
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
    pub password: Option<String>,
    pub created_by: Option<RecordId>,
    pub updated_by: Option<RecordId>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Table for Account {
    fn name() -> &'static str {
        TABLE_NAME
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AccountSignin {
    pub email: EmailAddress,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccountResp {
    pub account: Account,
    pub token: Uuid,
}
