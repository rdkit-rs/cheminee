use crate::indexing::index_manager::IndexManager;
use crate::rest_api::api::{IndexMeta, ListIndexResponseErr, ListIndexesResponse};
use crate::schema::LIBRARY;
use poem_openapi::payload::Json;

pub fn v1_list_indexes(index_manager: &IndexManager) -> ListIndexesResponse {
    let list_result = index_manager.list();
    if let Err(e) = list_result {
        return ListIndexesResponse::Err(Json(ListIndexResponseErr {
            error: format!("{:?}", e),
        }));
    }

    let index_metas = list_result
        .unwrap()
        .into_iter()
        .map(|x| {
            let index = index_manager.open(&x);
            let schema = match index {
                Ok(index) => {
                    let tantivy_schema = index.schema();
                    let mut schema_name = "";
                    for (k, v) in LIBRARY.clone().into_iter() {
                        if tantivy_schema == v {
                            schema_name = k
                        }
                    }
                    schema_name.to_string()
                }
                Err(e) => format!("error open index: {:?}", e),
            };
            IndexMeta { name: x, schema }
        })
        .collect();

    ListIndexesResponse::Ok(Json(index_metas))
}
