use poem_openapi::{payload::Json, ApiResponse, Object};

#[derive(ApiResponse)]
pub enum StandardizeResponse {
    #[oai(status = "200")]
    Ok(Json<Vec<StandardizedSmile>>),
}

#[derive(Object, Debug)]
pub struct StandardizedSmile {
    #[oai(skip_serializing_if_is_none)]
    pub smile: Option<String>,
    #[oai(skip_serializing_if_is_none)]
    pub error: Option<String>,
}
