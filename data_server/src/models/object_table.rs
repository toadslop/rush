use super::object_field::ObjectField;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectTable {
    pub published: bool,
    pub system: bool,
    pub id: String,
    pub object_fields: Vec<ObjectField>,
}
