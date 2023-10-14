use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use super::instance::Instance;

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
