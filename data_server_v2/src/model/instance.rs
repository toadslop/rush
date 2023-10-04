use std::{fmt::Display, ops::Deref};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Instance {
    pub name: String,
}

#[derive(Debug)]
pub struct InstanceName(String);

impl Deref for InstanceName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for InstanceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// TODO: eventually, we'll need to be able to handle any domain sent to this url
// We will need a mapping of urls to instance names so that we can identify
// which instance a domain is associated with

impl TryFrom<&str> for InstanceName {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut period_count: u8 = 0;
        let mut instance_name = None;
        for (idx, char) in value.chars().enumerate() {
            if char == '.' {
                if period_count == 0 {
                    instance_name = Some(InstanceName(value[0..idx].into()));
                }
                period_count += 1
            }
        }

        if period_count == 2 {
            return instance_name.ok_or("Failed to parse the host");
        }

        Err("Failed to parse the host")
    }
}
