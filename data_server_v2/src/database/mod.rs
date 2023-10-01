use include_dir::include_dir;
use include_dir::Dir;
use surrealdb::engine::any::connect;
use surrealdb::engine::any::Any;
use surrealdb::Surreal;

use crate::configuration::ConnectionType;
use crate::configuration::DatabaseSettings;

// pub static DB: Lazy<Surreal<Any>> = Lazy::new(Surreal::init); // TODO: need to get rid of singleton and implement a connection pool
pub static DB_QUERIES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/database/resources");

pub async fn init_db(settings: DatabaseSettings) -> anyhow::Result<Surreal<Any>> {
    let db: Surreal<Any> = connect(settings.connection.get_conn_string()).await?;

    if settings.connection == ConnectionType::InMemory {
        let query = DB_QUERIES
            .get_file("init-db.surql")
            .expect("Failed to find init-db script")
            .contents_utf8()
            .expect("Failed to extract contents of init-db script");
        db.query(query)
            .await
            .expect("Failed to run init_db script on database");
    };

    db.use_ns("root")
        .use_db("root")
        .await
        .expect("Could not use the root namespace or root db");

    Ok(db)
}
