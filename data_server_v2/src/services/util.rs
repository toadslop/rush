use actix_web::HttpResponse;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    message: String,
}

#[derive(Debug)]
pub struct HttpError(HttpResponse);

impl From<surrealdb::Error> for HttpError {
    fn from(value: surrealdb::Error) -> Self {
        match value {
            surrealdb::Error::Db(e) => match e {
                surrealdb::error::Db::FieldCheck { thing, value, field, check } => Self(HttpResponse::BadRequest().json(ErrorBody {
                    message: format!("type of value '{value} for {field} of entity '{thing}' should conform to the following type: '{check}'"),
                })),
                surrealdb::error::Db::FieldValue { 
                    thing,
                    value,
                    field,
                    check,
                } => Self(HttpResponse::BadRequest().json(ErrorBody {
                    message: format!("value '{value} for {field} of entity '{thing}' does not conform to the following expression: '{check}'"),
                })),
                surrealdb::error::Db::TxFailure => Self(HttpResponse::BadRequest().json(ErrorBody {
                    message: "Transaction failed".into(),
                })),
                surrealdb::error::Db::QueryNotExecuted => Self(HttpResponse::BadRequest().json(ErrorBody {
                    message: e.to_string(),
                })),
                _ => Self(HttpResponse::InternalServerError().json(ErrorBody {
                    message: "An unknown internal server error occurred".into(),
                })),
            },
            surrealdb::Error::Api(err) => match err {
                surrealdb::error::Api::Query(e) => Self(HttpResponse::BadRequest().json(ErrorBody {
                    message: e,
                })),
                _ => Self(HttpResponse::InternalServerError().json(ErrorBody {
                    message: "An unknown internal server error occurred".into(),
                })),
            },
        }
    }
}

impl HttpError {
    pub fn inter_inner(self) -> HttpResponse {
        self.0
    }
}

// Note to self: relying on surrealdb's assertions for validations will lead to the following problems:
// 1. difficulty providing user friendly error messages
// 2. only returns one error when possibly multiple fields have errors
// 3. cannot execute server side
// 4. cannot separate multiple validations, each with a distinct error message
// Possible solutions
// 1. Make PR directly to surrealdb to modify how errors are handled
// 2. implement my own validation system from scratch <--
//    maybe create a 'validations' table in surrealdb
//    generate assert clauses from the table
//    read the table to validate client side and server side
