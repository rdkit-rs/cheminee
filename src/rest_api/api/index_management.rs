use poem_openapi::{payload::Json, ApiResponse, Object};

use crate::{indexing::index_manager::IndexManager, schema::LIBRARY};

#[derive(ApiResponse)]
pub enum ListSchemasResponse {
    #[oai(status = "200")]
    Ok(Json<Vec<Schema>>),
}

#[derive(Object, Debug)]
pub struct Schema {
    pub name: String,
    pub schema: serde_json::Value,
}

#[derive(ApiResponse)]
pub enum ListIndexesResponse {
    #[oai(status = "200")]
    Ok(Json<Vec<IndexMeta>>),
    #[oai(status = "500")]
    Err(Json<ListIndexResponseErr>),
}

#[derive(Object)]
pub struct ListIndexResponseErr {
    pub error: String,
}

#[derive(ApiResponse)]
pub enum GetIndexesResponse {
    #[oai(status = "200")]
    Ok(Json<Vec<IndexMeta>>),
    #[oai(status = "400")]
    NotFound,
    #[oai(status = "500")]
    Err(Json<GetIndexesResponseError>),
}

#[derive(Object)]
pub struct GetIndexesResponseError {
    pub error: String,
}

#[derive(Object, Debug)]
pub struct IndexMeta {
    pub name: String,
    pub schema: String,
}

pub async fn v1_list_schemas() -> ListSchemasResponse {
    let schema_descriptions = LIBRARY
        .iter()
        .map(|(name, schema)| Schema {
            name: name.to_string(),
            schema: serde_json::to_value(schema).unwrap(),
        })
        .collect();

    ListSchemasResponse::Ok(Json(schema_descriptions))
}

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

#[allow(unused_variables)]
pub fn v1_get_index(index_manager: &IndexManager, index: String) -> GetIndexesResponse {
    let index = index_manager.open(&index);

    match index {
        Ok(index) => GetIndexesResponse::Ok(Json(vec![])),
        Err(e) => GetIndexesResponse::Err(Json(GetIndexesResponseError {
            error: format!("{}", e),
        })),
    }
}
