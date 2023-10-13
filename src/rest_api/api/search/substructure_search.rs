use poem_openapi::payload::Json;
use poem_openapi_derive::{ApiResponse, Object};

#[derive(ApiResponse)]
pub enum GetSubstructureSearchResponse {
    #[oai(status = "200")]
    Ok(Json<Vec<SubstructureSearchHit>>),
}

#[derive(Object)]
pub struct SubstructureSearchHit {
    pub extra_data: serde_json::Value,
    pub smiles: String,
    pub score: f32,
}
