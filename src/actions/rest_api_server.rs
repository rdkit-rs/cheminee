use crate::analysis::compound_processing::standardize_smiles;
use poem::{listener::TcpListener, Route, Server};
use poem_openapi::{payload::Json, ApiResponse, Object, OpenApi, OpenApiService};
use rayon::prelude::*;

pub const NAME: &'static str = "rest-api-server";
pub fn command() -> clap::Command<'static> {
    clap::Command::new("rest-api-server").subcommand(
        clap::Command::new("spec").arg(
            clap::Arg::new("output")
                .required(true)
                .short('d')
                .long("output")
                .takes_value(true),
        ),
    )
}

#[derive(ApiResponse)]
pub enum StandardizeResponse {
    #[oai(status = "200")]
    Ok(Json<Vec<Smile>>),
}

#[derive(Object)]
pub struct Smile {
    pub smile: String,
}

struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/standardize", method = "post")]
    async fn standardize(&self, mol: Json<Vec<Smile>>) -> StandardizeResponse {
        let standardized_smiles = mol
            .0
            // .into_iter()
            .into_par_iter()
            .map(|s| Smile {
                smile: standardize_smiles(&s.smile).as_smile(),
            })
            .collect::<Vec<_>>();

        StandardizeResponse::Ok(Json(standardized_smiles))
    }
}

fn api_service() -> OpenApiService<Api, ()> {
    OpenApiService::new(Api, "Cheminée", "1.0").server("http://localhost:3000/api")
}

async fn run_api_service() -> eyre::Result<()> {
    let api_service = api_service();
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

fn output_spec(dest: String) -> eyre::Result<()> {
    let api_service = api_service();

    let spec = api_service.spec();

    std::fs::write(dest, spec)?;

    Ok(())
}

pub async fn action(matches: &clap::ArgMatches) -> eyre::Result<()> {
    match matches.subcommand() {
        None => run_api_service().await?,
        Some(("spec", args)) => output_spec(args.value_of_t_or_exit("output"))?,
        Some((other, _args)) => Err(eyre::eyre!("can't handle {}", other))?,
    }

    Ok(())
}
