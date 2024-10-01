use crate::search::{QuerySearchHit, StructureSearchHit};
use poem_openapi::{payload::Json, ApiResponse, Object};
use tantivy::Opstamp;

// Response types
#[derive(ApiResponse, Debug)]
pub enum StandardizeResponse {
    #[oai(status = "200", content_type = "application/json")]
    Ok(Json<Vec<StandardizedSmiles>>),
}

#[derive(ApiResponse, Debug)]
pub enum ConvertedSmilesResponse {
    #[oai(status = "200", content_type = "application/json")]
    Ok(Json<Vec<ConvertedSmiles>>),
}

#[derive(ApiResponse, Debug)]
pub enum ConvertedMolBlockResponse {
    #[oai(status = "200", content_type = "application/json")]
    Ok(Json<Vec<ConvertedMolBlock>>),
}

#[derive(ApiResponse, Debug)]
pub enum GetIndexResponse {
    #[oai(status = "200", content_type = "application/json")]
    Ok(Json<crate::rest_api::api::IndexSchema>),
    #[oai(status = "400")]
    NotFound,
    #[oai(status = "500", content_type = "application/json")]
    Err(Json<crate::rest_api::api::GetIndexesResponseError>),
}

#[derive(ApiResponse)]
pub enum GetIndexesResponse {
    #[oai(status = "200", content_type = "application/json")]
    Ok(Json<Vec<crate::rest_api::api::IndexMeta>>),
    #[oai(status = "400")]
    NotFound,
    #[oai(status = "500", content_type = "application/json")]
    Err(Json<crate::rest_api::api::GetIndexesResponseError>),
}

#[derive(ApiResponse, Debug)]
pub enum ListIndexesResponse {
    #[oai(status = "200", content_type = "application/json")]
    Ok(Json<Vec<crate::rest_api::api::IndexMeta>>),
    #[oai(status = "500", content_type = "application/json")]
    Err(Json<crate::rest_api::api::ListIndexResponseErr>),
}

#[derive(ApiResponse, Debug)]
pub enum ListSchemasResponse {
    #[oai(status = "200", content_type = "application/json")]
    Ok(Json<Vec<crate::rest_api::api::Schema>>),
}

#[derive(ApiResponse, Debug)]
pub enum PostIndexesBulkIndexResponse {
    #[oai(status = "200", content_type = "application/json")]
    Ok(Json<crate::rest_api::api::PostIndexBulkResponseOk>),
    #[oai(status = "404")]
    IndexDoesNotExist,
    #[oai(status = "500", content_type = "application/json")]
    Err(Json<crate::rest_api::api::PostIndexBulkResponseError>),
}

#[derive(ApiResponse, Debug)]
pub enum DeleteIndexesBulkDeleteResponse {
    #[oai(status = "200", content_type = "application/json")]
    Ok(Json<crate::rest_api::api::DeleteIndexBulkResponseOk>),
    #[oai(status = "404")]
    IndexDoesNotExist,
    #[oai(status = "500", content_type = "application/json")]
    Err(Json<crate::rest_api::api::DeleteIndexBulkResponseError>),
}

#[derive(ApiResponse, Debug)]
pub enum PostIndexResponse {
    #[oai(status = "200", content_type = "application/json")]
    Ok(Json<crate::rest_api::api::IndexMeta>),
    #[oai(status = "404")]
    SchemaDoesNotExist,
    #[oai(status = "400")]
    IndexExists,
    #[oai(status = "500", content_type = "application/json")]
    ServerErr(Json<crate::rest_api::api::CreateIndexError>),
}

#[derive(ApiResponse, Debug)]
pub enum DeleteIndexResponse {
    #[oai(status = "200", content_type = "application/json")]
    Ok(Json<crate::rest_api::api::IndexMeta>),
    #[oai(status = "404")]
    IndexDoesNotExist,
    #[oai(status = "500", content_type = "application/json")]
    Err(Json<crate::rest_api::api::DeleteIndexError>),
}

#[derive(ApiResponse, Debug)]
pub enum GetQuerySearchResponse {
    #[oai(status = "200", content_type = "application/json")]
    Ok(Json<Vec<QuerySearchHit>>),
    #[oai(status = "404")]
    IndexDoesNotExist,
    #[oai(status = "500", content_type = "application/json")]
    Err(Json<crate::rest_api::api::QueryResponseError>),
}

#[derive(ApiResponse, Debug)]
pub enum GetStructureSearchResponse {
    #[oai(status = "200", content_type = "application/json")]
    Ok(Json<Vec<StructureSearchHit>>),
    #[oai(status = "404")]
    IndexDoesNotExist,
    #[oai(status = "500", content_type = "application/json")]
    Err(Json<crate::rest_api::api::StructureResponseError>),
}

// Relevant structs
#[derive(Object, Debug)]
pub struct StandardizedSmiles {
    #[oai(skip_serializing_if_is_none)]
    pub smiles: Option<String>,
    #[oai(skip_serializing_if_is_none)]
    pub error: Option<String>,
}

#[derive(Object, Debug)]
pub struct ConvertedSmiles {
    #[oai(skip_serializing_if_is_none)]
    pub smiles: Option<String>,
    #[oai(skip_serializing_if_is_none)]
    pub error: Option<String>,
}

#[derive(Object, Debug)]
pub struct ConvertedMolBlock {
    #[oai(skip_serializing_if_is_none)]
    pub mol_block: Option<String>,
    #[oai(skip_serializing_if_is_none)]
    pub error: Option<String>,
}

#[derive(Object, Debug)]
pub struct BulkRequest {
    pub docs: Vec<crate::rest_api::api::BulkRequestDoc>,
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

#[derive(Object, Debug)]
pub struct GetIndexesResponseError {
    pub error: String,
}

#[derive(Object, Debug, serde::Serialize)]
pub struct IndexMeta {
    pub name: String,
    pub schema: String,
}

#[derive(Object, Debug)]
pub struct IndexSchema {
    pub index: String,
    pub schema: serde_json::Value,
}

#[derive(Object, Debug)]
pub struct ListIndexResponseErr {
    pub error: String,
}

#[derive(Object, Debug)]
pub struct PostIndexBulkResponseError {
    pub error: String,
}

#[derive(Object, Debug)]
pub struct PostIndexBulkResponseOk {
    pub statuses: Vec<crate::rest_api::api::PostIndexBulkResponseOkStatus>,
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
    pub statuses: Vec<crate::rest_api::api::DeleteIndexBulkResponseOkStatus>,
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

#[derive(Object, Debug)]
pub struct QueryResponseError {
    pub error: String,
}

#[derive(Object, Debug)]
pub struct StructureResponseError {
    pub error: String,
}

#[derive(ApiResponse, Debug)]
pub enum MergeSegmentsResponse {
    #[oai(status = "200", content_type = "application/json")]
    Ok(Json<String>),
    #[oai(status = "404")]
    IndexDoesNotExist,
    #[oai(status = "500")]
    MergeFailed(Json<String>),
}
