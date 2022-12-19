use crate::analysis::compound_processing::standardize_smiles;
use poem::{handler, listener::TcpListener, post, test::TestClient, Route, Server};
use poem_openapi::{payload::Json, ApiResponse, Object, OpenApi, OpenApiService};
use rayon::prelude::*;
use rdkit::ROMol;

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

#[derive(Object, Debug)]
pub struct Smile {
    pub smile: String,
}

#[derive(ApiResponse)]
pub enum StandardizeResponse {
    #[oai(status = "200")]
    Ok(Json<Vec<StandardizedSmile>>),
}

#[derive(Object, Debug)]
pub struct StandardizedSmile {
    #[oai(skip_serializing_if_is_none)]
    pub smile: Option<String>,
    #[oai(skip_serializing_if_is_none)]
    pub error: Option<String>,
}

struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/standardize", method = "post")]
    async fn standardize(&self, mol: Json<Vec<Smile>>) -> StandardizeResponse {
        let standardized_smiles = mol
            .0
            .into_par_iter()
            .map(|s| {
                let standardize = standardize_smiles(&s.smile);

                match standardize {
                    Ok(romol) => StandardizedSmile {
                        smile: Some(romol.as_smile()),
                        error: None,
                    },
                    Err(e) => StandardizedSmile {
                        smile: Some(s.smile),
                        error: Some(e.to_string()),
                    },
                }
            })
            .collect::<Vec<_>>();

        StandardizeResponse::Ok(Json(standardized_smiles))
    }
}

fn api_service(hostname: &str, port: i16) -> OpenApiService<Api, ()> {
    OpenApiService::new(Api, "Cheminée", "1.0").server(format!("http://{hostname}:{port}/api"))
}

async fn run_api_service() -> eyre::Result<()> {
    let hostname = "127.0.0.1";
    let port = 3000;

    let api_service = api_service(hostname, port);
    let ui = api_service.swagger_ui();

    let spec = api_service.spec();
    Server::new(TcpListener::bind(format!("{hostname}:{port}")))
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
    let api_service = api_service("127.0.0.1", 3000);

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

// {
// "smile": "CC=CO"
// }
// ]'
// [{"smile":"CCC=O"}]%

#[handler]
async fn index() -> StandardizeResponse {
    let smiles = Json(vec![Smile {
        smile: "CC=CO".to_string(), // smile:  "CCC=O".to_string(), -answer
    }]);
    Api.standardize(smiles).await
}

#[tokio::test]
async fn test_poem() {
    let app = Route::new().at("/", post(index));
    let client = TestClient::new(app);

    let resp = client.post("/").send().await;

    resp.assert_status_is_ok();

    let json = resp.json().await;
    let json_value = json.value();
    // json_value.object().get("smile").assert_string("CCC=O");
    json_value
        .array()
        .iter()
        .map(|value| value.object().get("smile"))
        .collect::<Vec<_>>()
        .first()
        .expect("first_value")
        .assert_string("CCC=O");
    println!("{:?}", json_value);
    // TestJsonValue(Array([Object({"smile": String("CCC=O")})]))
    //     resp.assert_text("CCC=O").await;

    println!("lllla")
}