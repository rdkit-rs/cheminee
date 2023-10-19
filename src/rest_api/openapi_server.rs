use std::path::PathBuf;

use poem::{listener::TcpListener, EndpointExt, Route, Server};
use poem_openapi::{
    param::{Path, Query},
    payload::Json,
    ContactObject, OpenApiService,
};
use poem_openapi_derive::OpenApi;

use crate::{
    indexing::index_manager::IndexManager,
    rest_api::{
        api::{
            v1_get_index, v1_index_search_substructure, v1_list_indexes, v1_list_schemas,
            v1_post_index, v1_post_index_bulk, v1_standardize, BulkRequest, GetIndexesResponse,
            GetSubstructureSearchResponse, ListIndexesResponse, ListSchemasResponse,
            PostIndexResponse, PostIndexesBulkIndexResponse, StandardizeResponse,
        },
        models::Smile,
    },
};

const API_PREFIX: &str = "/api";

pub fn api_service(
    server_url: &str,
    api_prefix: &str,
    indexes_root: PathBuf,
    create_storage_dir_if_missing: bool,
) -> eyre::Result<OpenApiService<Api, ()>> {
    let api = Api {
        index_manager: IndexManager::new(indexes_root, create_storage_dir_if_missing)?,
    };
    let openapi_service = OpenApiService::new(api, "Cheminée", "1.0")
        .server(format!("{}{}", server_url, api_prefix))
        .description("Cheminée: The Chemical Structure Search Engine")
        .contact(ContactObject::new().url("https://github.com/rdkit-rs/cheminee"));
    Ok(openapi_service)
}

pub async fn run_api_service(
    bind: &str,
    server_url: &str,
    index_path: PathBuf,
    create_storage_dir_if_missing: bool,
) -> eyre::Result<()> {
    let api_service = api_service(
        server_url,
        API_PREFIX,
        index_path,
        create_storage_dir_if_missing,
    )?;
    let ui = api_service.swagger_ui();

    let spec = api_service.spec();

    let logging_middleware = poem::middleware::Tracing;

    Server::new(TcpListener::bind(bind))
        .run(
            Route::new()
                .at(
                    "/api/v1/openapi.json",
                    poem::endpoint::make_sync(move |_| spec.clone()),
                )
                .nest(API_PREFIX, api_service)
                .nest("/", ui)
                .with(logging_middleware),
        )
        .await?;

    Ok(())
}

pub struct Api {
    pub index_manager: IndexManager,
}

#[OpenApi]
impl Api {
    #[oai(path = "/v1/standardize", method = "post")]
    /// Pass a list of SMILES through fragment_parent, uncharger, and canonicalization routines
    pub async fn v1_standardize(&self, mol: Json<Vec<Smile>>) -> StandardizeResponse {
        v1_standardize(mol).await
    }

    #[oai(path = "/v1/schemas", method = "get")]
    /// List schemas available for creating indexes
    pub async fn v1_list_schemas(&self) -> ListSchemasResponse {
        v1_list_schemas().await
    }

    #[oai(path = "/v1/indexes", method = "get")]
    /// List indexes
    pub async fn v1_list_indexes(&self) -> ListIndexesResponse {
        v1_list_indexes(&self.index_manager)
    }

    #[oai(path = "/v1/indexes/:index", method = "get")]
    /// Get extended information about an index
    pub async fn v1_get_index(&self, index: Path<String>) -> GetIndexesResponse {
        v1_get_index(&self.index_manager, index.to_string())
    }

    // v1/indexes/inventory_items_v1?schema=v1_descriptors
    #[oai(path = "/v1/indexes/:index", method = "post")]
    /// Create an index
    pub async fn v1_post_index(
        &self,
        index: Path<String>,
        schema: Query<String>,
        sort_by: Query<Option<String>>,
    ) -> PostIndexResponse {
        v1_post_index(
            &self.index_manager,
            index.to_string(),
            schema.0,
            sort_by.0.as_deref(),
        )
    }

    #[oai(path = "/v1/indexes/:index/bulk_index", method = "post")]
    /// Index a list of SMILES and associated, free-form JSON attributes
    /// which are indexed and searchable
    pub async fn v1_post_indexes_bulk_index(
        &self,
        index: Path<String>,
        bulk_request: Json<BulkRequest>,
    ) -> PostIndexesBulkIndexResponse {
        v1_post_index_bulk(&self.index_manager, index.to_string(), bulk_request.0).await
    }

    #[oai(path = "/v1/indexes/:index/search/substructure", method = "get")]
    /// Perform substructure search against index
    pub async fn v1_index_search_substructure(
        &self,
        index: Path<String>,
        q: Query<String>,
    ) -> GetSubstructureSearchResponse {
        v1_index_search_substructure(index.to_string(), Some(q.0))
    }
}

pub fn output_spec(server_url: &str, output: &str) -> eyre::Result<()> {
    let api_service = api_service(
        server_url,
        API_PREFIX,
        std::path::PathBuf::from("/tmp/cheminee"),
        false,
    )
    .unwrap();

    let spec = api_service.spec();

    std::fs::write(output, spec)?;

    Ok(())
}
