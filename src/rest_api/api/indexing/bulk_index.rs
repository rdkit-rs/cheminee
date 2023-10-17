use poem_openapi::payload::Json;
use poem_openapi_derive::{ApiResponse, Enum, Object};

use crate::{indexing::index_manager::IndexManager, rest_api::api::IndexMeta, schema::LIBRARY};
// v1_post_indexes_bulk_index
#[derive(ApiResponse)]
pub enum PostIndexesBulkIndexResponse {
    #[oai(status = "200")]
    Ok(Json<PostIndexBulkResponseOk>),
    #[oai(status = "404")]
    SchemaDoesNotExist,
    #[oai(status = "500")]
    Err(Json<PostIndexBulkResponseError>),
}

#[derive(Object, Debug)]
pub struct BulkRequest {
    pub docs: Vec<BulkRequestIndexAction>,
}

#[derive(Object, Debug)]
pub struct BulkRequestIndexAction {
    pub smile: String,
    pub extra_data: Option<serde_json::Value>,
}

#[derive(Object, Debug)]
pub struct PostIndexBulkResponseOk {
    pub statuses: String,
}

#[derive(Object, Debug)]
pub struct PostIndexBulkResponseError {
    pub error: String,
}

pub fn v1_post_index_bulk(
    index_manager: &IndexManager,
    index: String,
    bulk_request: BulkRequest,
) -> PostIndexesBulkIndexResponse {
    unimplemented!()
}
