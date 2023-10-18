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
    pub query: String,
}

pub fn v1_index_search_substructure(
    index: String,
    q: Option<String>,
) -> GetSubstructureSearchResponse {
    let q_str = format!("{:?}", q);

    GetSubstructureSearchResponse::Ok(Json(vec![SubstructureSearchHit {
        extra_data: serde_json::json!({"hi": "mom", "index": index}),
        smiles: ":)".to_string(),
        score: 100.00,
        query: q_str,
    }]))
}
