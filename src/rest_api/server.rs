use poem::{listener::TcpListener, Route, Server};
use poem_openapi::{ContactObject, OpenApiService};

use crate::rest_api::Api;

pub fn api_service(server_url: &str) -> OpenApiService<Api, ()> {
    OpenApiService::new(Api, "Cheminée", "1.0")
        .server(format!("{}/api/v1", server_url))
        .description("Cheminée: The Chemical Structure Search Engine")
        .contact(ContactObject::new().url("https://github.com/rdkit-rs/cheminee"))
}

pub async fn run_api_service(bind: &str, server_url: &str) -> eyre::Result<()> {
    let api_service = api_service(server_url);
    let ui = api_service.swagger_ui();

    let spec = api_service.spec();
    Server::new(TcpListener::bind(bind))
        .run(
            Route::new()
                .at(
                    "/api/v1/openapi.json",
                    poem::endpoint::make_sync(move |_| spec.clone()),
                )
                .nest("/api/v1", api_service)
                .nest("/", ui),
        )
        .await?;

    Ok(())
}
