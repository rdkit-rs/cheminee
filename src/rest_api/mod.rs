pub mod api;
pub mod models;
pub mod server;

use clap::Arg;
use models::Smile;
use poem_openapi::{payload::Json, OpenApi};

pub const NAME: &str = "rest-api-server";
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

pub struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/standardize", method = "post")]
    async fn standardize(&self, mol: Json<Vec<Smile>>) -> api::standardize::StandardizeResponse {
        api::standardize::standardize(mol).await
    }

    #[oai(path = "/schemas", method = "get")]
    async fn list_schemas(&self) -> api::index_management::ListSchemaResponse {
        api::index_management::list_schemas().await
    }
}

fn output_spec(server_url: &str, output: &str) -> eyre::Result<()> {
    let api_service = server::api_service(server_url);

    let spec = api_service.spec();

    std::fs::write(output, spec)?;

    Ok(())
}

pub async fn action(matches: &clap::ArgMatches) -> eyre::Result<()> {
    match matches.subcommand() {
        None => {
            let bind: &String = matches.get_one("bind").unwrap();
            let server_url: &String = matches.get_one("server-url").unwrap();
            server::run_api_service(bind, server_url).await?
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

// {
// "smile": "CC=CO"
// }
// ]'
// [{"smile":"CCC=O"}]%
