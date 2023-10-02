use include_dir::include_dir;
use include_dir::Dir;
use surrealdb::engine::any::connect;
use surrealdb::engine::any::Any;
use surrealdb::Surreal;

use crate::configuration::ConnectionType;
use crate::configuration::DatabaseSettings;

// pub static DB: Lazy<Surreal<Any>> = Lazy::new(Surreal::init); // TODO: need to get rid of singleton and implement a connection pool
pub static DB_QUERIES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/database/resources");

#[tracing::instrument(name = "Initializing the database")]
pub async fn init_db(settings: DatabaseSettings) -> anyhow::Result<Surreal<Any>> {
    tracing::info!("Initializing the database");
    tracing::debug!("Attempting to connect to the database");
    let db: Surreal<Any> = connect(settings.connection.get_conn_string()).await?;
    tracing::debug!("Connection success");

    if settings.connection == ConnectionType::InMemory {
        tracing::debug!("Initializing in-memory database. This should be used for testing only.");
        let query = DB_QUERIES
            .get_file("init-db.surql")
            .expect("Failed to find init-db script")
            .contents_utf8()
            .expect("Failed to extract contents of init-db script");

        tracing::trace!("Running initialization query: {query}");

        db.query(query)
            .await
            .expect("Failed to run init_db script on database");
        tracing::trace!("Initialization query success")
    };

    tracing::debug!("Accessing to root ns and root db");
    db.use_ns("root")
        .use_db("root")
        .await
        .expect("Could not use the root namespace or root db");
    tracing::debug!("Accessing success");

    tracing::info!("Initialation success");
    Ok(db)
}
