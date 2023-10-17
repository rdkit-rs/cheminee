// use poem_openapi::payload::Json;
// use poem_openapi_derive::{ApiResponse, Object};
//
// use crate::{indexing::index_manager::IndexManager, rest_api::api::IndexMeta, schema::LIBRARY};
//
// #[derive(ApiResponse)]
// pub enum PostIndexBulkResponse {
//     #[oai(status = "200")]
//     Ok(Json<PostIndexBulkResponseOk>),
//     #[oai(status = "404")]
//     SchemaDoesNotExist,
//     #[oai(status = "500")]
//     Err(Json<PostIndexBulkResponseError>),
// }
//
// #[derive(Object, Debug)]
// pub struct PostIndexBulkResponseOk {
//     pub statuses: String,
// }
//
// #[derive(Object, Debug)]
// pub struct PostIndexBulkResponseError {
//     pub error: String,
// }
//
// pub fn v1_post_index_bulk(
//     index_manager: &IndexManager,
//     index: String,
//     schema_name: String,
// ) -> PostIndexBulkResponse {
//     let schema = match LIBRARY.get(schema_name.as_str()) {
//         Some(schema) => schema,
//         None => return PostIndexBulkResponse::SchemaDoesNotExist,
//     };
//
//     let tantivy_index = index_manager.create(&index, schema, false, None);
//
//     match tantivy_index {
//         Ok(_) => PostIndexBulkResponse::Ok(Json(IndexMeta {
//             name: index,
//             schema: schema_name,
//         })),
//         Err(e) => PostIndexBulkResponse::Err(Json(PostIndexBulkResponseError {
//             error: e.to_string(),
//         })),
//     }
// }
