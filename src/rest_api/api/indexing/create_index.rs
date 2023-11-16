use crate::indexing::index_manager::IndexManager;
use crate::rest_api::api::{CreateIndexError, IndexMeta, PostIndexResponse};
use crate::schema::LIBRARY;
use poem_openapi::payload::Json;

pub fn v1_post_index(
    index_manager: &IndexManager,
    index: String,
    schema_name: String,
    sort_by: Option<&str>,
) -> PostIndexResponse {
    let schema = match LIBRARY.get(schema_name.as_str()) {
        Some(schema) => schema,
        None => return PostIndexResponse::SchemaDoesNotExist,
    };

    let tantivy_index = index_manager.create(&index, schema, false, sort_by);

    match tantivy_index {
        Ok(_) => PostIndexResponse::Ok(Json(IndexMeta {
            name: index,
            schema: schema_name,
        })),
        Err(e) => PostIndexResponse::Err(Json(CreateIndexError {
            error: e.to_string(),
        })),
    }
}
