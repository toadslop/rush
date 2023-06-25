use actix_web::{delete, get, patch, post, put, web, Result};

use crate::{models::object_table::ObjectTable, pipe::PipeManager, DB};

pub fn api(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(
                web::scope("/batch")
                    .service(get_batch)
                    .service(post_batch)
                    .service(update_batch)
                    .service(delete_batch)
                    .service(upsert_batch),
            )
            .service(
                web::resource("/objects/publish")
                    .wrap(PipeManager)
                    .route(web::post().to(publish_object)),
            )
            .service(
                web::resource("/{module}/{resource}")
                    .wrap(PipeManager)
                    .route(web::get().wrap(PipeManager).to(get_resource_page)),
            )
            .service(get_resource_instance)
            .service(post_resource)
            .service(update_resource)
            .service(upsert_resource)
            .service(delete_resource),
    );
}

async fn publish_object(info: web::Json<Vec<ObjectTable>>) -> Result<String> {
    println!("PUBLISH OBJECT");

    for object_table in info.iter() {
        let table_name = object_table.id.split(':').last().unwrap();
        println!("TABLE: {table_name}");
        let mut query = format!("DEFINE TABLE {table_name};\n");

        for object_field in object_table.object_fields.iter() {
            let field_name = object_field.id.split(':').last().unwrap();
            println!("FIELD: {field_name}");
            query.push_str(&format!("DEFINE FIELD {field_name} ON {table_name};\n"))
        }
        println!("\n{query}");
        DB.use_ns("rush_root_ns")
            .use_db("rush_root_db")
            .await
            .unwrap();

        let res = DB.query(query).await;
        println!("GOT RESPONSE");
        println!("{:?}", res);
    }
    Ok("Ok".into())
}

async fn get_resource_page(info: web::Path<(String, String)>) -> Result<String> {
    println!("HERE");
    let info = info.into_inner();
    Ok(format!("Module {}, resource: {}", info.0, info.1))
}

// #[get("/{module}/{resource}")]
// async fn get_resource_page(info: web::Path<(String, String)>) -> Result<String> {
//     let info = info.into_inner();
//     Ok(format!("Module {}, resource: {}", info.0, info.1))
// }

#[post("/{module}/{resource}")]
async fn post_resource(info: web::Path<(String, String)>) -> Result<String> {
    let info = info.into_inner();
    Ok(format!("Module {}, resource: {}", info.0, info.1))
}

#[patch("/{module}/{resource}")]
async fn update_resource(info: web::Path<(String, String)>) -> Result<String> {
    let info = info.into_inner();
    Ok(format!("Module {}, resource: {}", info.0, info.1))
}

#[put("/{module}/{resource}/{identifier}")]
async fn upsert_resource(info: web::Path<(String, String, String)>) -> Result<String> {
    let info = info.into_inner();
    Ok(format!("Module {}, resource: {}", info.0, info.1))
}

#[delete("/{module}/{resource}/{identifier}")]
async fn delete_resource(info: web::Path<(String, String)>) -> Result<String> {
    let info = info.into_inner();
    Ok(format!("Module {}, resource: {}", info.0, info.1))
}

#[get("/{module}/{resource}/{identifier}")]
async fn get_resource_instance(info: web::Path<(String, String, String)>) -> Result<String> {
    let info = info.into_inner();
    Ok(format!(
        "Module {}, resource: {}, action: {}",
        info.0, info.1, info.2
    ))
}

#[get("/{module}/{resource}")]
async fn get_batch(info: web::Path<(String, String)>) -> Result<String> {
    let info = info.into_inner();
    Ok(format!("BATCH -- Module {}, resource: {}", info.0, info.1))
}

#[post("/{module}/{resource}")]
async fn post_batch(info: web::Path<(String, String)>) -> Result<String> {
    let info = info.into_inner();
    Ok(format!("Module {}, resource: {}", info.0, info.1))
}

#[patch("/{module}/{resource}")]
async fn update_batch(info: web::Path<(String, String)>) -> Result<String> {
    let info = info.into_inner();
    Ok(format!("Module {}, resource: {}", info.0, info.1))
}

#[put("/{module}/{resource}")]
async fn upsert_batch(info: web::Path<(String, String)>) -> Result<String> {
    let info = info.into_inner();
    Ok(format!("Module {}, resource: {}", info.0, info.1))
}

#[delete("/{module}/{resource}")]
async fn delete_batch(info: web::Path<(String, String)>) -> Result<String> {
    let info = info.into_inner();
    Ok(format!("Module {}, resource: {}", info.0, info.1))
}
