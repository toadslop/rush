use include_dir::include_dir;
use include_dir::Dir;
use surrealdb::engine::any::connect;
use surrealdb::engine::any::Any;
use surrealdb::opt::auth::Root;
use surrealdb::opt::capabilities::Capabilities;
use surrealdb::opt::Config;
use surrealdb::Surreal;

use crate::configuration::db::ConnectionType;
use crate::configuration::db::DatabaseSettings;
use crate::configuration::Environment;
use crate::database::util::Transaction;

pub static DB_QUERIES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/database/resources");

#[tracing::instrument(name = "Initializing the database")]
pub async fn init_db(
    settings: DatabaseSettings,
    app_env: &Environment,
) -> anyhow::Result<Surreal<Any>> {
    tracing::info!("Initializing the database");
    tracing::debug!("Attempting to connect to the database");

    let db = connect_to_root_db(&settings).await?;

    if settings.connection == ConnectionType::InMemory || *app_env == Environment::Dev {
        tracing::debug!("Initializing in-memory database. This should be used for testing only.");

        let mut init = db
            .query("RETURN $INITIALIZED;")
            .await
            .expect("Initialized query failed");

        let init: Option<bool> = init.take(0).expect("Got nothing from query");

        if !init.unwrap_or(false) {
            let tx: String = DB_QUERIES
                .files()
                .map(|f| f.contents_utf8().expect("Failed to read surql file"))
                .fold(Transaction::new(), |tx, q| tx.add_query(q))
                .build();

            tracing::trace!("Running initialization query: {tx}");

            let mut res = db
                .query(tx)
                .await
                .map_err(|e| format!("Failed to run init_db script on database: {e}"))
                .unwrap();
            let errors = res.take_errors();
            dbg!(errors);

            tracing::trace!("Initialization query success")
        }
    };

    db.invalidate()
        .await
        .expect("Failed to invalidate initial connection.");

    tracing::info!("Initialation success");
    Ok(db)
}

#[tracing::instrument(name = "Connect to database")]
pub async fn connect_to_root_db(settings: &DatabaseSettings) -> anyhow::Result<Surreal<Any>> {
    tracing::debug!("Attempting to connect to the database");
    let capabilities = Capabilities::all();
    let config = Config::default().capabilities(capabilities).user(Root {
        username: "root",
        password: "root", // TODO: handle through configuration
    });

    let db: Surreal<Any> = connect((settings.connection.get_conn_string(), config)).await?;
    tracing::debug!("Connection success");

    tracing::debug!("Accessing to root ns and root db");

    db.use_ns("root")
        .use_db("root")
        .await
        .expect("Could not use the root namespace or root db");

    db.signin(Root {
        password: "root", // TODO: configuration by env variable
        username: "root",
    })
    .await
    .expect("Failed to login to the database");

    tracing::debug!("Accessing success");

    Ok(db)
}
