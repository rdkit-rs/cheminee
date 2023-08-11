use crate::search::compound_processing::standardize_smiles;
use clap::Arg;
use poem::{handler, listener::TcpListener, Route, Server};
use poem_openapi::{payload::Json, ApiResponse, ContactObject, Object, OpenApi, OpenApiService};
use rayon::prelude::*;

pub const NAME: &'static str = "rest-api-server";
pub fn command() -> clap::Command {
    clap::Command::new("rest-api-server").arg(
        Arg::new("bind").num_args(1).required(false).short('b').long("bind").default_value("localhost:3000")
    ).arg(
        Arg::new("server-url").num_args(1).required(false).short('u').long("server-url").default_value("http://localhost:3000")
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

fn api_service(server_url: &str) -> OpenApiService<Api, ()> {
    OpenApiService::new(Api, "Cheminée", "1.0")
        .server(format!("{}/api/v1", server_url))
        .description("Cheminée: The Chemical Structure Search Engine")
        .contact(ContactObject::new().url("https://github.com/rdkit-rs/cheminee"))
}

async fn run_api_service(bind: &str, server_url: &str) -> eyre::Result<()> {
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

fn output_spec(server_url: &String, output: &String) -> eyre::Result<()> {
    let api_service = api_service(server_url);

    let spec = api_service.spec();

    std::fs::write(output, spec)?;

    Ok(())
}

pub async fn action(matches: &clap::ArgMatches) -> eyre::Result<()> {
    match matches.subcommand() {
        None => {
            let bind: &String = matches.get_one("bind").unwrap();
            let server_url: &String = matches.get_one("server-url").unwrap();
            run_api_service(bind, server_url).await?
        }
        Some(("spec", args)) => {
            let server_url = matches.get_one("server-url").unwrap();
            let output = args.get_one::<String>("output").unwrap();
            output_spec(server_url, output)?
        }
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
    let app = Route::new().at("/", poem::post(index));
    let client = poem::test::TestClient::new(app);

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
