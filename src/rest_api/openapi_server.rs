use crate::indexing::index_manager::IndexManager;

use poem::{listener::TcpListener, EndpointExt, Route, Server};
use poem_openapi::{ContactObject, OpenApiService};
use std::path::PathBuf;

use super::api::ApiV1;

pub const API_PREFIX: &str = "/api";

pub fn api_service(server_url: &str, api_prefix: &str) -> eyre::Result<OpenApiService<ApiV1, ()>> {
    let api = ApiV1::default();
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
    let index_manager = IndexManager::new(index_path, create_storage_dir_if_missing)?;
    let api_service = api_service(server_url, API_PREFIX)?;
    let ui = api_service.swagger_ui();

    let spec = api_service.spec();

    let logging_middleware = poem::middleware::Tracing;

    let app = Route::new()
        .at(
            "/api/v1/openapi.json",
            poem::endpoint::make_sync(move |_| spec.clone()),
        )
        .nest(API_PREFIX, api_service)
        .nest("/", ui)
        .with(logging_middleware)
        .data(index_manager);

    Server::new(TcpListener::bind(bind))
        .run_with_graceful_shutdown(
            app,
            async move {
                let _ = tokio::signal::ctrl_c().await;
            },
            Some(tokio::time::Duration::from_secs(5)),
        )
        .await?;

    Ok(())
}

pub fn output_spec(server_url: &str, output: &str) -> eyre::Result<()> {
    let api_service = api_service(server_url, API_PREFIX)?;

    let spec = api_service.spec();

    std::fs::write(output, spec)?;

    Ok(())
}
