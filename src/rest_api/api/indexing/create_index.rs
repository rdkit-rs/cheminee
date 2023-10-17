use poem_openapi::payload::Json;
use poem_openapi_derive::{ApiResponse, Object};

use crate::{indexing::index_manager::IndexManager, rest_api::api::IndexMeta, schema::LIBRARY};

#[derive(ApiResponse)]
pub enum PostIndexResponse {
    #[oai(status = "200")]
    Ok(Json<IndexMeta>),
    #[oai(status = "404")]
    SchemaDoesNotExist,
    #[oai(status = "500")]
    Err(Json<CreateIndexError>),
}

#[derive(Object, Debug)]
pub struct CreateIndexError {
    pub error: String,
}

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
