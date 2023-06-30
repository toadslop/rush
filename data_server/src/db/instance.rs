use surrealdb::{opt::auth::Root, Surreal};

use crate::db::KvInfo;

static ROOT_NS: &str = "rush_root";

pub enum AppType {
    Root,
    Virtual(String),
}

pub async fn initialize_instance<T>(
    db: &Surreal<T>,
    app_type: AppType,
) -> Result<(), shared::anyhow::Error>
where
    T: surrealdb::Connection,
{
    db.signin(Root {
        username: "root", // TODO: environment variable
        password: "root", // TODO: environment variable
    })
    .await?;

    let mut res = db.query("INFO FOR KV").await?;
    let kv_info: Option<KvInfo> = res.take(0)?;
    let kv_info = kv_info.unwrap(); // TODO: treat as error
    println!("{:?}", kv_info);
    let name = match &app_type {
        AppType::Root => ROOT_NS,
        AppType::Virtual(name) => name,
    };

    println!("Checking database");
    let ns_name = &format!("{name}_ns");
    if kv_info.ns.get(ns_name).is_none() {
        println!("Creating database for {}...", name);
        let res = db.query(get_init_string(name)).await?;
        res.check()?;
        println!("Successfully initialized");
    } else {
        println!("Application {name} already exists");
    }

    db.use_ns(ns_name).use_db(format!("{name}_db")).await?;
    let res = db.query(SAMPLE_DATA).await?;
    println!("{:?}", res);

    // TODO: initialize db in namespace
    // TODO: create user table
    // TODO: create scope
    // TODO: create roles table
    // TODO: create permissions table
    // TODO: create default admin user
    // TODO: define permission checking function
    // TODO: define objects table
    println!("{:?}", kv_info);
    Ok(())
}

fn get_init_string(app_name: &str) -> String {
    format!(
        r#"
    BEGIN TRANSACTION;
    DEFINE NS {app_name}_ns;
    USE NS {app_name}_ns;
    DEFINE DB {app_name}_db;
    USE DB {app_name}_db;
    
    DEFINE TABLE object_table SCHEMAFUL;
    DEFINE FIELD published ON object_table
        TYPE bool
        VALUE $value OR false;
    DEFINE FIELD system on object_table
        TYPE bool
        VALUE $value OR false
        PERMISSIONS
            FOR create, update NONE;
    DEFINE FIELD settings.* ON object_table TYPE object;
    DEFINE FIELD name ON object_table TYPE string;

    DEFINE TABLE object_field SCHEMAFUL;
    DEFINE FIELD settings.* ON object_field TYPE object;
    DEFINE FIELD name ON object_field TYPE string;

    CREATE object_table:object_table SET system = true;
    CREATE object_field:object_field;
    RELATE object_table:object_table -> has_field -> object_field:object_field;

    DEFINE LOGIN rush_root_db_user ON DATABASE PASSWORD 'test';

    DEFINE INDEX unique_relationships
        ON TABLE has_field 
        COLUMNS in, out UNIQUE;

    DEFINE EVENT publish_object ON object_table WHEN $after.published = true THEN (
        http::post("http://127.0.0.1:8080/api/objects/publish", (SELECT *, ->has_field.out.* AS object_fields FROM object_table WHERE id = $after.id))
    );
    COMMIT TRANSACTION;
    "#
    )
}

static SAMPLE_DATA: &str = "
CREATE object_table:shipment SET name = \"Shipment\";
CREATE object_table:port SET name = \"Port\";

CREATE object_field:origin SET name = \"Origin\";
RELATE object_table:shipment -> has_field -> object_field:origin;

DEFINE INDEX unique_relationships 
    ON TABLE has_field 
    COLUMNS in, out UNIQUE;

CREATE object_field:origin_port SET name = \"Origin Port\";
RELATE object_table:shipment -> has_field -> object_field:origin_port;

CREATE object_field:unlocode SET name =\"UNLOCODE\";
RELATE object_table:port -> has_field -> object_field:unlocode;
";
// DEFINE TABLE user SCHEMAFULL
// PERMISSIONS
//     FOR select, update WHERE id = $auth.id,
//     FOR create, delete NONE;
// DEFINE FIELD email ON user TYPE string ASSERT is::email($value);
//     DEFINE FIELD password ON user TYPE string;
//     DEFINE FIELD settings.* ON user TYPE object;

//     DEFINE INDEX idx_user_email ON user COLUMNS email UNIQUE;

//     DEFINE SCOPE guest
//         SESSION 15m
//         SIGNIN ( SELECT * FROM user WHERE user = "guest" LIMIT 1 );

//     DEFINE SCOPE user
//         SESSION 15m
//         SIGNUP ( CREATE user SET email = $email, password = crypto::argon2::generate($password) )
//         SIGNIN ( SELECT * FROM user WHERE user = $email AND crypto::argon2::compare(pass, $password) );

// DONT DELETE
// SELECT *, ->has_field.out.* AS object_fields FROM object_table WHERE id = object_table:shipments;
