use poem::handler;
use poem_openapi::{payload::Json, ApiResponse, Object};
use rayon::prelude::*;

#[derive(ApiResponse)]
pub enum ListSchemaResponse {
    #[oai(status = "200")]
    Ok(Json<Vec<Schema>>),
}

#[derive(Object, Debug)]
pub struct Schema {
    pub name: String,
}

pub async fn list_schemas() -> ListSchemaResponse {
    // ListSchemasResponse::Ok(Json(vec![]))
    unimplemented!("bang")
}
