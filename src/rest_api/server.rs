use std::path::PathBuf;

use poem::{listener::TcpListener, EndpointExt, Route, Server};
use poem_openapi::{ContactObject, OpenApiService};
use tantivy::Index;
use tokio::sync::Mutex;

use crate::{indexing::index_manager::IndexManager, rest_api::Api};

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
