use crate::rest_api::api::{ListSchemasResponse, Schema};
use crate::schema::LIBRARY;
use poem_openapi::payload::Json;

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
