use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct EmailAddress(pub String);

impl AsRef<str> for EmailAddress {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl Display for EmailAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_owned())
    }
}
