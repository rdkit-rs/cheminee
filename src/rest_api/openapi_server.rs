use std::path::PathBuf;

use poem::{listener::TcpListener, Route, Server};
use poem_openapi::{
    param::{Path, Query},
    payload::Json,
    ContactObject, OpenApiService,
};
use poem_openapi_derive::OpenApi;
use tokio::sync::Mutex;

use crate::{
    indexing::index_manager::IndexManager,
    rest_api::{
        api::{
            v1_get_index, v1_index_search_substructure, v1_list_indexes, v1_list_schemas,
            v1_standardize, GetIndexesResponse, GetSubstructureSearchResponse, ListIndexesResponse,
            ListSchemasResponse, StandardizeResponse,
        },
        models::Smile,
    },
};

pub fn api_service(
    server_url: &str,
    indexes_root: PathBuf,
    create_storage_dir_if_missing: bool,
) -> eyre::Result<OpenApiService<Api, ()>> {
    let api = Api {
        index_manager: Mutex::new(IndexManager::new(
            indexes_root,
            create_storage_dir_if_missing,
        )?),
    };
    let openapi_service = OpenApiService::new(api, "Cheminée", "1.0")
        .server(server_url)
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
    let api_service = api_service(server_url, index_path, create_storage_dir_if_missing)?;
    let ui = api_service.swagger_ui();

    let spec = api_service.spec();
    Server::new(TcpListener::bind(bind))
        .run(
            Route::new()
                .at(
                    "/api/v1/openapi.json",
                    poem::endpoint::make_sync(move |_| spec.clone()),
                )
                .nest("/api", api_service)
                .nest("/", ui),
        )
        .await?;

    Ok(())
}

pub struct Api {
    pub index_manager: Mutex<IndexManager>,
}

#[OpenApi]
impl Api {
    #[oai(path = "/v1/standardize", method = "post")]
    pub async fn v1_standardize(&self, mol: Json<Vec<Smile>>) -> StandardizeResponse {
        v1_standardize(mol).await
    }

    #[oai(path = "/v1/schemas", method = "get")]
    pub async fn v1_list_schemas(&self) -> ListSchemasResponse {
        v1_list_schemas().await
    }

    #[oai(path = "/v1/indexes", method = "get")]
    pub async fn v1_list_indexes(&self) -> ListIndexesResponse {
        let manager = self.index_manager.lock().await;

        v1_list_indexes(&manager)
    }

    #[oai(path = "/v1/indexes/:index", method = "get")]
    #[allow(unused_variables)]
    pub async fn v1_get_index(&self, index: Path<String>) -> GetIndexesResponse {
        let index_manager = self.index_manager.lock().await;

        v1_get_index(&index_manager, index.to_string())
    }

    // v1/indexes/inventory_items_v1/search/substructure?q=1234
    #[oai(path = "/v1/indexes/:index/search/substructure", method = "get")]
    #[allow(unused_variables)]
    pub async fn v1_index_search_substructure(
        &self,
        index: Path<String>,
        q: Query<Option<String>>,
    ) -> GetSubstructureSearchResponse {
        v1_index_search_substructure(index.to_string(), q.0)
    }
}
