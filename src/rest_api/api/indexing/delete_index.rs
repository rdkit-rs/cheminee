use crate::indexing::index_manager::IndexManager;
use crate::rest_api::api::{DeleteIndexError, DeleteIndexResponse, IndexMeta};
use crate::schema::LIBRARY;
use poem_openapi::payload::Json;

pub fn v1_delete_index(index_manager: &IndexManager, index_name: String) -> DeleteIndexResponse {
    let index = match index_manager.open(&index_name) {
        Ok(index) => index,
        Err(e) => {
            return DeleteIndexResponse::Err(Json(DeleteIndexError {
                error: e.to_string(),
            }))
        }
    };

    let schema = index.schema();
    let mut schema_name = "";
    for (k, v) in LIBRARY.clone().into_iter() {
        if schema == v {
            schema_name = k
        }
    }

    let delete_operation = index_manager.delete(&index_name);

    match delete_operation {
        Ok(_) => DeleteIndexResponse::Ok(Json(IndexMeta {
            name: index_name,
            schema: schema_name.to_string(),
        })),
        Err(e) => DeleteIndexResponse::Err(Json(DeleteIndexError {
            error: e.to_string(),
        })),
    }
}
