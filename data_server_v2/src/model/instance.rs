use serde::{Deserialize, Serialize};
use std::{fmt::Display, ops::Deref};
use surrealdb::opt::RecordId;

use super::{account::Account, CreateTable, Table};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInstanceDto {
    pub name: String,
    pub account_id: String,
}

impl CreateTable for CreateInstanceDb {
    fn id(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateInstanceDb {
    name: String,
    account: RecordId,
    id: RecordId,
}

impl From<CreateInstanceDto> for CreateInstanceDb {
    fn from(value: CreateInstanceDto) -> Self {
        Self {
            name: value.name.to_owned(),
            account: RecordId::from((Account::name(), value.account_id.as_ref())),
            id: RecordId::from((Instance::name(), value.name.as_ref())),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Instance {
    pub name: Option<String>,
    pub account: Option<RecordId>,
    pub id: Option<RecordId>,
}

impl Table for Instance {
    fn name() -> &'static str {
        "instance"
    }
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
