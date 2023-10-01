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
    if let Err(err) = DB.connect(settings.connection.get_conn_string()).await {
        match &err {
            surrealdb::Error::Db(_err) => Err(err),
            surrealdb::Error::Api(inner) => match inner {
                surrealdb::error::Api::AlreadyConnected => Ok(()),
                _ => Err(err),
            },
        }
    } else {
        Ok(())
    }
    .expect("Failed to connect to database");

    if settings.connection == ConnectionType::InMemory {
        let query = DB_QUERIES
            .get_file("init-db.surql")
            .expect("Failed to find init-db script")
            .contents_utf8()
            .expect("Failed to extract contents of init-db script");
        DB.query(query)
            .await
            .expect("Failed to run init_db script on database");
    };

    DB.use_ns("root")
        .use_db("root")
        .await
        .expect("Could not use the root namespace or root db");

    Ok(())
}
