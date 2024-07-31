use poem_openapi::{payload::Json, ApiResponse, Object};

#[derive(ApiResponse, Debug)]
pub enum StandardizeResponse {
    #[oai(status = "200")]
    Ok(Json<Vec<StandardizedSmiles>>),
}

#[derive(Object, Debug)]
pub struct StandardizedSmiles {
    #[oai(skip_serializing_if_is_none)]
    pub smiles: Option<String>,
    #[oai(skip_serializing_if_is_none)]
    pub error: Option<String>,
}
