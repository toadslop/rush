use super::instance::Instance;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAccount {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Account {
    pub id: Option<Thing>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub confirmed: Option<bool>,
    pub instances: Option<Vec<Instance>>,
}

impl From<CreateAccount> for Account {
    fn from(value: CreateAccount) -> Self {
        Self {
            id: Some(Thing::from((value.email.as_str(), value.email.as_str()))),
            email: Some(value.email),
            name: None,
            confirmed: None,
            instances: Some(vec![]),
        }
    }
}
