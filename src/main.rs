use cheminee::{command_line, rest_api};
use clap::*;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    if let Some(rust_debug) = std::env::var_os("RUST_DEBUG") {
        tracing_subscriber::fmt()
            .with_env_filter(
                rust_debug
                    .to_str()
                    .ok_or(eyre::eyre!("could not convert RUST_DEBUG to str"))?,
            )
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter("poem=info,tokio-runtime-worker=debug")
            .init();
    }

    let app = Command::new("cheminee")
        .subcommand_required(true)
        .subcommand(command_line::indexing::bulk_index::command())
        .subcommand(command_line::indexing::bulk_delete::command())
        .subcommand(command_line::indexing::create_index::command())
        .subcommand(command_line::indexing::delete_index::command())
        .subcommand(command_line::indexing::index_sdf::command())
        .subcommand(command_line::pubchem::fetch_pubchem::command())
        .subcommand(command_line::pubchem::stream_pubchem_sdf::command())
        .subcommand(command_line::search::basic_search::command())
        .subcommand(command_line::search::substructure_search::command())
        .subcommand(command_line::search::superstructure_search::command())
        .subcommand(command_line::search::identity_search::command())
        .subcommand(command_line::search::similarity_search::command())
        .subcommand(rest_api::command());

    let matches = app.get_matches();
    let matches = match matches.subcommand().unwrap() {
        (command_line::indexing::bulk_index::NAME, matches) => {
            command_line::indexing::bulk_index::action(matches)
        }
        (command_line::indexing::bulk_delete::NAME, matches) => {
            command_line::indexing::bulk_delete::action(matches)
        }
        (command_line::indexing::create_index::NAME, matches) => {
            command_line::indexing::create_index::action(matches)
        }
        (command_line::indexing::delete_index::NAME, matches) => {
            command_line::indexing::delete_index::action(matches)
        }
        (command_line::indexing::index_sdf::NAME, matches) => {
            command_line::indexing::index_sdf::action(matches)
        }
        (command_line::pubchem::fetch_pubchem::NAME, matches) => {
            command_line::pubchem::fetch_pubchem::action(matches).await
        }
        (command_line::pubchem::stream_pubchem_sdf::NAME, matches) => {
            command_line::pubchem::stream_pubchem_sdf::action(matches)
        }
        (command_line::search::basic_search::NAME, matches) => {
            command_line::search::basic_search::action(matches)
        }
        (command_line::search::substructure_search::NAME, matches) => {
            command_line::search::substructure_search::action(matches)
        }
        (command_line::search::superstructure_search::NAME, matches) => {
            command_line::search::superstructure_search::action(matches)
        }
        (command_line::search::identity_search::NAME, matches) => {
            command_line::search::identity_search::action(matches)
        }
        (command_line::search::similarity_search::NAME, matches) => {
            command_line::search::similarity_search::action(matches)
        }
        (rest_api::NAME, matches) => rest_api::action(matches).await,
        (unknown, _) => panic!("🤨: {}", unknown),
    };

    matches?;

    Ok(())
}
