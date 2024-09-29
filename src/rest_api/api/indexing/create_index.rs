use crate::indexing::index_manager::IndexManager;
use crate::rest_api::api::{CreateIndexError, IndexMeta, PostIndexResponse};
use crate::schema::LIBRARY;
use poem_openapi::payload::Json;

pub fn v1_post_index(
    index_manager: &IndexManager,
    index: String,
    schema_name: String,
) -> PostIndexResponse {
    let schema = match LIBRARY.get(schema_name.as_str()) {
        Some(schema) => schema,
        None => return PostIndexResponse::SchemaDoesNotExist,
    };

    match index_manager.exists(&index) {
        Ok(Some(_)) => return PostIndexResponse::IndexExists,
        Ok(None) => (),
        Err(e) => {
            return PostIndexResponse::ServerErr(Json(CreateIndexError {
                error: e.to_string(),
            }))
        }
    }

    match index_manager.create(&index, schema, false) {
        Ok(_) => PostIndexResponse::Ok(Json(IndexMeta {
            name: index,
            schema: schema_name,
        })),
        Err(e) => PostIndexResponse::ServerErr(Json(CreateIndexError {
            error: e.to_string(),
        })),
    }
}
