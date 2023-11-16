use crate::indexing::index_manager::IndexManager;
use crate::rest_api::api::{GetIndexResponse, GetIndexesResponseError, IndexSchema};
use poem_openapi::payload::Json;

pub fn v1_get_index(index_manager: &IndexManager, index_name: String) -> GetIndexResponse {
    let index = index_manager.open(&index_name);

    match index {
        Ok(index) => {
            let schema = serde_json::to_value(index.schema()).unwrap();

            GetIndexResponse::Ok(Json(IndexSchema {
                index: index_name,
                schema,
            }))
        }
        Err(e) => GetIndexResponse::Err(Json(GetIndexesResponseError {
            error: format!("{}", e),
        })),
    }
}
