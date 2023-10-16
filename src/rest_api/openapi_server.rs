use std::path::PathBuf;

use poem::{listener::TcpListener, Route, Server};
use poem_openapi::{param::Path, payload::Json, ContactObject, OpenApiService};
use poem_openapi_derive::OpenApi;
use tokio::sync::Mutex;

use crate::{
    indexing::index_manager::IndexManager,
    rest_api::{
        api,
        api::{
            index_management::{GetIndexesResponseError, IndexMeta, ListIndexResponseErr},
            search::substructure_search::SubstructureSearchHit,
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
    index_manager: Mutex<IndexManager>,
}

#[OpenApi]
impl Api {
    #[oai(path = "/v1/standardize", method = "post")]
    async fn v1_standardize(&self, mol: Json<Vec<Smile>>) -> api::standardize::StandardizeResponse {
        api::standardize::standardize(mol).await
    }

    #[oai(path = "/v1/schemas", method = "get")]
    async fn v1_list_schemas(&self) -> api::index_management::ListSchemasResponse {
        api::index_management::list_schemas().await
    }

    #[oai(path = "/v1/indexes", method = "get")]
    async fn v1_list_indexes(&self) -> api::index_management::ListIndexesResponse {
        let manager = self.index_manager.lock().await;

        let list_result = manager.list();
        if let Err(e) = list_result {
            return api::index_management::ListIndexesResponse::Err(Json(ListIndexResponseErr {
                error: format!("{:?}", e),
            }));
        }

        let index_metas = list_result
            .unwrap()
            .into_iter()
            .map(|x| IndexMeta { name: x })
            .collect();

        api::index_management::ListIndexesResponse::Ok(Json(index_metas))
    }

    #[oai(path = "/v1/indexes/:index", method = "get")]
    #[allow(unused_variables)]
    async fn v1_get_index(&self, index: Path<String>) -> api::index_management::GetIndexesResponse {
        let index_manager = self.index_manager.lock().await;
        let index = index_manager.open(&index);

        match index {
            Ok(index) => api::index_management::GetIndexesResponse::Ok(Json(vec![])),
            Err(e) => {
                api::index_management::GetIndexesResponse::Err(Json(GetIndexesResponseError {
                    error: format!("{}", e),
                }))
            }
        }
    }

    // v1/indexes/inventory_items_v1/search/substructure?q=1234
    #[oai(path = "/v1/indexes/:index/search/substructure", method = "get")]
    #[allow(unused_variables)]
    async fn v1_index_search_substructure(
        &self,
        index: Path<String>,
    ) -> api::search::substructure_search::GetSubstructureSearchResponse {
        api::search::substructure_search::GetSubstructureSearchResponse::Ok(Json(vec![
            SubstructureSearchHit {
                extra_data: serde_json::json!({"hi": "mom", "index": index.to_string()}),
                smiles: ":)".to_string(),
                score: 100.00,
            },
        ]))
    }
}
