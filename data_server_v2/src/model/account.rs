use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAccount {
    pub email: String,
    pub password: String,
}
