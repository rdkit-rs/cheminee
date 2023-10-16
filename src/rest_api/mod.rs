pub mod api;
pub mod models;
pub mod openapi_server;

use clap::{Arg, ArgAction};
use models::Smile;
use poem_openapi::{param::Path, payload::Json, OpenApi};
use tokio::sync::Mutex;

use crate::{
    indexing::index_manager::IndexManager,
    rest_api::api::{
        index_management::{GetIndexesResponseError, IndexMeta, ListIndexResponseErr},
        search::substructure_search::SubstructureSearchHit,
    },
};

pub const NAME: &str = "rest-api-server";
pub fn command() -> clap::Command {
    clap::Command::new("rest-api-server").arg(
        Arg::new("bind").num_args(1).required(false).short('b').long("bind").default_value("localhost:3000")
    ).arg(
        Arg::new("server-url").num_args(1).required(false).short('u').long("server-url").default_value("http://localhost:3000")
    ).arg(
        Arg::new("index-storage-directory").num_args(1).required(false).short('s').long("index-storage-directory").default_value("/tmp/cheminee")
    ).arg(
        Arg::new("index-storage-directory-create-if-missing").required(false).long("index-storage-directory-create-if-missing").action(ArgAction::SetTrue)
    ).subcommand(
        clap::Command::new("spec").arg(
            clap::Arg::new("output")
                .help("Write openapi JSON specific to destination. Useful for building Cheminee client implementations.")
                .required(true)
                .short('o')
                .long("output")
                .num_args(1),
        ),
    )
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

fn output_spec(server_url: &str, output: &str) -> eyre::Result<()> {
    let api_service =
        openapi_server::api_service(server_url, std::path::PathBuf::from("/tmp/cheminee"), false)
            .unwrap();

    let spec = api_service.spec();

    std::fs::write(output, spec)?;

    Ok(())
}

pub async fn action(matches: &clap::ArgMatches) -> eyre::Result<()> {
    match matches.subcommand() {
        None => {
            let bind: &String = matches.get_one("bind").unwrap();
            let server_url: &String = matches.get_one("server-url").unwrap();
            let index_storage_directory: &String =
                matches.get_one("index-storage-directory").unwrap();
            let index_storage_directory_create_if_missing: bool =
                matches.get_flag("index-storage-directory-create-if-missing");

            openapi_server::run_api_service(
                bind,
                server_url,
                index_storage_directory.into(),
                index_storage_directory_create_if_missing,
            )
            .await?
        }
        Some(("spec", args)) => {
            let server_url: &String = matches.get_one("server-url").unwrap();
            let output: &String = args.get_one::<String>("output").unwrap();
            output_spec(server_url, output)?
        }
        Some((other, _args)) => Err(eyre::eyre!("can't handle {}", other))?,
    }

    Ok(())
}
