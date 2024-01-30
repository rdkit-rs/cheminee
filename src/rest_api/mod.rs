pub mod api;
pub mod models;
pub mod openapi_server;

use clap::{Arg, ArgAction};

pub const NAME: &str = "rest-api-server";
pub fn command() -> clap::Command {
    clap::Command::new("rest-api-server").arg(
        Arg::new("bind").num_args(1).required(false).short('b').long("bind").default_value("localhost:4001")
    ).arg(
        Arg::new("server-url").num_args(1).required(false).short('u').long("server-url").default_value("http://localhost:4001")
    ).arg(
        Arg::new("index-storage-directory").num_args(1).required(false).short('s').long("index-storage-directory").default_value("/tmp/cheminee")
    ).arg(
        Arg::new("index-storage-directory-create-if-missing").required(false).long("index-storage-directory-create-if-missing").action(ArgAction::SetTrue)
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

pub async fn action(matches: &clap::ArgMatches) -> eyre::Result<()> {
    match matches.subcommand() {
        None => {
            let bind: &String = matches.get_one("bind").unwrap();
            let server_url: &String = matches.get_one("server-url").unwrap();
            let index_storage_directory: &String =
                matches.get_one("index-storage-directory").unwrap();
            let index_storage_directory_create_if_missing: bool =
                matches.get_flag("index-storage-directory-create-if-missing");

            openapi_server::run_api_service(
                bind,
                server_url,
                index_storage_directory.into(),
                index_storage_directory_create_if_missing,
            )
            .await?
        }
        Some(("spec", args)) => {
            let server_url: &String = matches.get_one("server-url").unwrap();
            let output: &String = args.get_one::<String>("output").unwrap();
            openapi_server::output_spec(server_url, output)?
        }
        Some((other, _args)) => Err(eyre::eyre!("can't handle {}", other))?,
    }

    Ok(())
}
