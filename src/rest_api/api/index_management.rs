use poem_openapi::{payload::Json, ApiResponse, Object};

use crate::schema::LIBRARY;

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
}

pub async fn list_schemas() -> ListSchemasResponse {
    let schema_descriptions = LIBRARY
        .iter()
        .map(|(name, schema)| Schema {
            name: name.to_string(),
            schema: serde_json::to_value(&schema).unwrap(),
        })
        .collect();

    ListSchemasResponse::Ok(Json(schema_descriptions))
}
