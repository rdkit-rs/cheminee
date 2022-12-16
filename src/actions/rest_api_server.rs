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

#[derive(ApiResponse)]
pub enum StandardizeResponse {
    #[oai(status = "200")]
    Ok(Json<Vec<Smile>>),
}

#[derive(Object, Debug)]
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
            .into_par_iter()
            .map(|s| Smile {
                smile: standardize_smiles(&s.smile)
                    .map(|s| s.as_smile())
                    .unwrap_or_else(|_| format!("could not process smile: {}", s.smile)),
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

// #[handler]
// pub async fn graphql_handler(schema: Data<&AppSchema>, req:  Json<Vec<Smile>>) -> StandardizeResponse {
//     Json(schema.execute(req.0).await)
// }

// {
// "smile": "CC=CO"
// }
// ]'
// [{"smile":"CCC=O"}]%

#[handler]
async fn index() -> Json<Vec<Smile>> {
    // fn index() -> Json<Vec<Smile>> {
    Json(vec![Smile {
        // smile:  "CC=CO".to_string(),
        smile: "CCC=O".to_string(),
    }])
}
// #[handler]
// async fn index(mol: Json<Vec<Smile>>) -> String  {
//     standardize_smiles(mol.0.first().unwrap().clone().smile.as_str()).as_smile()
// }
//async fn index(mol: Json<Vec<Smile>>) -> Vec<String> {
//     let sm = mol.0.first().unwrap().clone();
//     let standardized_smiles = standardize_smiles(sm.smile.as_str()).as_smile();
//     let standardized_smiles = mol
//       .0
//       .into_par_iter()
//       .map(|s| standardize_smiles(s.smile.as_str()).clone().as_smile()).collect::<Vec<String>>();

//standardized_smiles
//     // poem_openapi::payload::Json(standardized_smiles)
// }

#[tokio::test]
async fn test_poem() {
    // let app = Route::new().at("/", index);

    // let app = Route::new().at("/", post(serde_json::from_str(index)));
    // println!("{:?}", index.0.first().unwrap().clone());
    let app = Route::new().at("/", post(index));
    // let app = Route::new().at("/", index);
    let client = TestClient::new(app);
    //
    // let resp = client.get("/").send().await;
    //
    // // check the status code
    //     resp.assert_status_is_ok();
    // // check the json
    //     let json = resp.json().await;
    //     let json_value = json.value();
    //     println!("{:?}", json_value);
    //

    use serde_json::Value;
    let schema: Smile = Smile {
        smile: r#"[{ "smile": "CC=CO"}]"#.to_string(),
    };
    // let schema: Smile = Smile { smile: "[{ \"smile\": \"CC=CO\"}]".to_string() } ;
    // let mut object: Value = serde_json::from_str(schema).unwrap();
    let resp = client
        .post("/")
        // .post("/")
        // .post("/")
        // .data(vec![schema])  // <<<----- here
        // .body(r#"{"query": "{ health { status } }" }"#)
        //     .body(r#"[{ "smile": "CC=CO"}]"#)
        .send()
        .await;

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
    // check the body string
    //     resp.assert_text("CCC=O").await;
    println!("lllla")
}

extern crate reqwest;
use reqwest::header;
use reqwest::Client;
#[tokio::test]
async fn response_test() {
    let mut headers = header::HeaderMap::new();
    headers.insert("accept", "application/json".parse().unwrap());
    headers.insert("Content-Type", "application/json".parse().unwrap());

    // let client = reqwest::blocking::Client::new();
    let client = reqwest::Client::new();
    let res = client
        .post("http://localhost:3000/api/standardize")
        .headers(headers)
        // .query()
        .body(
            r#"
[
 {
 "smile": "CC=CO"
 }
]
"#,
        )
        .send()
        .await
        .unwrap();
    // .text();
    println!("{:?}", res);
    println!("{:?}", res.url());
    // Response { url: Url { scheme: "http", cannot_be_a_base: false, username: "", password: None, host: Some(Domain("localhost")), port: Some(3000), path: "/api/standardize", query: None, fragment: None }, status: 200, headers: {"content-type": "application/json; charset=utf-8", "content-length": "19", "date": "Thu, 15 Dec 2022 22:28:30 GMT"} }

    // Ok(())
}
