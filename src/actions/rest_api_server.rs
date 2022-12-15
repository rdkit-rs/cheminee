use crate::analysis::compound_processing::standardize_smiles;
use poem::{listener::TcpListener, Route, Server, post, handler, test::TestClient};
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
    Json( vec! [Smile {
        smile:  "CC=CO".to_string(),
        // smile:  "XXX".to_string(),
    }])
}


#[tokio::test]
async fn test_poem() {
    // let app = Route::new().at("/", index);

    let app = Route::new().at("/", post(index));
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
    let schema = "[{ \"smile\": \"CC=CO\"}]";
    let mut object: Value = serde_json::from_str(schema).unwrap();
    let resp = client
        // .post("/")
        .post("/")
        .data(object)  // <<<----- here
        // .body(r#"{"query": "{ health { status } }" }"#)
        //     .body(r#"[{ "smile": "CC=CO"}]"#)
        .send()
        .await;


    resp.assert_status_is_ok();
// check the body string

    resp.assert_text("CCC=O").await;
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
    let res = client.post("http://localhost:3000/api/standardize")
        .headers(headers)
        // .query()
        .body(r#"
[
 {
 "smile": "CC=CO"
 }
]
"#
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