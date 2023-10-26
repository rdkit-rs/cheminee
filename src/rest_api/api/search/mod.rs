mod substructure_search;

use crate::search::StructureSearchHit;
use poem_openapi::payload::Json;
use poem_openapi_derive::{ApiResponse, Object};
pub use substructure_search::*;

#[derive(ApiResponse)]
pub enum GetStructureSearchResponse {
    #[oai(status = "200")]
    Ok(Json<Vec<StructureSearchHit>>),
    #[oai(status = "404")]
    IndexDoesNotExist,
    #[oai(status = "500")]
    Err(Json<StructureResponseError>),
}

#[derive(Object, Debug)]
pub struct StructureResponseError {
    pub error: String,
}

#[derive(Object, Debug)]
pub struct StructureSearchResponseError {
    pub error: String,
}
