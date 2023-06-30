use std::collections::HashMap;

use include_dir::{include_dir, Dir};
use serde::Deserialize;
use shared::anyhow::Ok;
use std::path::Path;
use surrealdb::{opt::auth::Root, Surreal};

pub mod admin;
mod initializers;
pub mod instance;

static INITIALIZERS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/db/assets");

#[derive(Debug, Deserialize)]
struct KvInfo {
    pub ns: HashMap<String, String>,
}
