use poem::{listener::TcpListener, Route, Server};
use poem_openapi::{payload::Json as OpenapiJson, ApiResponse, Object, OpenApi, OpenApiService};

pub const NAME: &'static str = "rest-api-server";
pub fn command() -> clap::Command<'static> {
    clap::Command::new("rest-api-server")
}

#[derive(ApiResponse)]
pub enum StandardizeResponse {
    #[oai(status = "200")]
    Ok(OpenapiJson<Vec<Smile>>),
}

#[derive(Object)]
pub struct Smile {
    pub smile: String,
}

struct Api;

#[OpenApi]
impl Api {
    // curl -XPOST -d'{"smile": "abcd"}' http://localhost:3000/api/standardize
    #[oai(path = "/standardize", method = "post")]
    async fn standardize(&self, mol: OpenapiJson<Vec<Smile>>) -> StandardizeResponse {
        let standardized_smiles = mol
            .0
            .into_iter()
            .map(|s| Smile {
                smile: s.smile.to_uppercase(),
            })
            .collect::<Vec<_>>();

        StandardizeResponse::Ok(OpenapiJson(standardized_smiles))
    }
}

pub async fn action(matches: &clap::ArgMatches) -> eyre::Result<()> {
    let api_service =
        OpenApiService::new(Api, "Cheminée", "1.0").server("http://localhost:3000/api");
    let ui = api_service.swagger_ui();

    let spec = api_service.spec();
    Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(
            Route::new()
                .at(
                    "/openapi.json",
                    poem::endpoint::make_sync(move |_| spec.clone()),
                )
                .nest("/api", api_service)
                .nest("/", ui),
        )
        .await?;

    Ok(())
}
