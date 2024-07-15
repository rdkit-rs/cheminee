use poem_openapi::{payload::Json, ApiResponse, Object};
use tantivy::Opstamp;

#[derive(ApiResponse)]
pub enum GetIndexResponse {
    #[oai(status = "200")]
    Ok(Json<IndexSchema>),
    #[oai(status = "400")]
    NotFound,
    #[oai(status = "500")]
    Err(Json<GetIndexesResponseError>),
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

#[derive(ApiResponse)]
pub enum ListIndexesResponse {
    #[oai(status = "200")]
    Ok(Json<Vec<IndexMeta>>),
    #[oai(status = "500")]
    Err(Json<ListIndexResponseErr>),
}

#[derive(ApiResponse)]
pub enum ListSchemasResponse {
    #[oai(status = "200")]
    Ok(Json<Vec<Schema>>),
}

#[derive(ApiResponse)]
pub enum PostIndexesBulkIndexResponse {
    #[oai(status = "200")]
    Ok(Json<PostIndexBulkResponseOk>),
    #[oai(status = "404")]
    IndexDoesNotExist,
    #[oai(status = "500")]
    Err(Json<PostIndexBulkResponseError>),
}

#[derive(ApiResponse)]
pub enum DeleteIndexesBulkDeleteResponse {
    #[oai(status = "200")]
    Ok(Json<DeleteIndexBulkResponseOk>),
    #[oai(status = "404")]
    IndexDoesNotExist,
    #[oai(status = "500")]
    Err(Json<DeleteIndexBulkResponseError>),
}

#[derive(ApiResponse)]
pub enum PostIndexResponse {
    #[oai(status = "200")]
    Ok(Json<IndexMeta>),
    #[oai(status = "404")]
    SchemaDoesNotExist,
    #[oai(status = "500")]
    Err(Json<CreateIndexError>),
}

#[derive(ApiResponse)]
pub enum DeleteIndexResponse {
    #[oai(status = "200")]
    Ok(Json<IndexMeta>),
    #[oai(status = "404")]
    IndexDoesNotExist,
    #[oai(status = "500")]
    Err(Json<DeleteIndexError>),
}

#[derive(Object, Debug)]
pub struct BulkRequest {
    pub docs: Vec<BulkRequestDoc>,
}

#[derive(Object, Debug)]
pub struct BulkRequestDoc {
    pub smiles: String,
    /// This value can store an arbitrary JSON object like '{}'
    pub extra_data: Option<serde_json::Value>,
}

#[derive(Object, Debug)]
pub struct CreateIndexError {
    pub error: String,
}

#[derive(Object, Debug)]
pub struct DeleteIndexError {
    pub error: String,
}

#[derive(Object)]
pub struct GetIndexesResponseError {
    pub error: String,
}

#[derive(Object, Debug)]
pub struct IndexMeta {
    pub name: String,
    pub schema: String,
}

#[derive(Object, Debug)]
pub struct IndexSchema {
    pub index: String,
    pub schema: serde_json::Value,
}

#[derive(Object)]
pub struct ListIndexResponseErr {
    pub error: String,
}

#[derive(Object, Debug)]
pub struct PostIndexBulkResponseError {
    pub error: String,
}

#[derive(Object, Debug)]
pub struct PostIndexBulkResponseOk {
    pub statuses: Vec<PostIndexBulkResponseOkStatus>,
}

#[derive(Object, Debug)]
pub struct PostIndexBulkResponseOkStatus {
    pub opcode: Option<Opstamp>,
    pub error: Option<String>,
}

#[derive(Object, Debug)]
pub struct DeleteIndexBulkResponseError {
    pub error: String,
}

#[derive(Object, Debug)]
pub struct DeleteIndexBulkResponseOk {
    pub statuses: Vec<DeleteIndexBulkResponseOkStatus>,
}

#[derive(Object, Debug)]
pub struct DeleteIndexBulkResponseOkStatus {
    pub opcode: Option<Opstamp>,
    pub error: Option<String>,
}

#[derive(Object, Debug)]
pub struct Schema {
    pub name: String,
    pub schema: serde_json::Value,
}
