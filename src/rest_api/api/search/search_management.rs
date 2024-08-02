use crate::search::{QuerySearchHit, StructureSearchHit};
use poem_openapi::{payload::Json, ApiResponse, Object};

#[derive(ApiResponse, Debug)]
pub enum GetQuerySearchResponse {
    #[oai(status = "200")]
    Ok(Json<Vec<QuerySearchHit>>),
    #[oai(status = "404")]
    IndexDoesNotExist,
    #[oai(status = "500")]
    Err(Json<QueryResponseError>),
}

#[derive(ApiResponse, Debug)]
pub enum GetStructureSearchResponse {
    #[oai(status = "200")]
    Ok(Json<Vec<StructureSearchHit>>),
    #[oai(status = "404")]
    IndexDoesNotExist,
    #[oai(status = "500")]
    Err(Json<StructureResponseError>),
}

#[derive(Object, Debug)]
pub struct QueryResponseError {
    pub error: String,
}

#[derive(Object, Debug)]
pub struct StructureResponseError {
    pub error: String,
}
