use include_dir::include_dir;
use include_dir::Dir;
use once_cell::sync::Lazy;
use surrealdb::engine::any::Any;
use surrealdb::Surreal;

use crate::configuration::ConnectionType;
use crate::configuration::DatabaseSettings;

pub static DB: Lazy<Surreal<Any>> = Lazy::new(Surreal::init);
pub static DB_QUERIES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/database/resources");

pub async fn init_db(settings: DatabaseSettings) -> anyhow::Result<()> {
    DB.connect(settings.connection.get_conn_string())
        .await
        .expect("Failed to connect to database");

    if settings.connection == ConnectionType::InMemory {
        let query = DB_QUERIES
            .get_file("./init_db.surql")
            .expect("Failed to find init_db script")
            .contents_utf8()
            .expect("Failed to extract contents of init_db script");
        DB.query(query)
            .await
            .expect("Failed to run init_db script on database");
    };

    Ok(())
}
