#![deny(clippy::print_stdout)]

use cheminee::{command_line, rest_api};
use clap::*;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    if std::env::var_os("RUST_LOG").is_some() {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter("info").init();
    }

    let app = Command::new("cheminee")
        .subcommand_required(true)
        .subcommand(command_line::indexing::bulk_index::command())
        .subcommand(command_line::indexing::bulk_delete::command())
        .subcommand(command_line::indexing::create_index::command())
        .subcommand(command_line::indexing::delete_index::command())
        .subcommand(command_line::indexing::index_sdf::command())
        .subcommand(command_line::indexing::merge_segments::command())
        .subcommand(command_line::pubchem::fetch_pubchem::command())
        .subcommand(command_line::pubchem::stream_pubchem_sdf::command())
        .subcommand(command_line::search::basic_search::command())
        .subcommand(command_line::search::basic_search_count::command())
        .subcommand(command_line::search::substructure_search::command())
        .subcommand(command_line::search::superstructure_search::command())
        .subcommand(command_line::search::similarity_search::command())
        .subcommand(command_line::search::identity_search::command())
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
        (command_line::indexing::merge_segments::NAME, matches) => {
            command_line::indexing::merge_segments::action(matches)
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
        (command_line::search::basic_search_count::NAME, matches) => {
            command_line::search::basic_search_count::action(matches)
        }
        (command_line::search::substructure_search::NAME, matches) => {
            command_line::search::substructure_search::action(matches)
        }
        (command_line::search::superstructure_search::NAME, matches) => {
            command_line::search::superstructure_search::action(matches)
        }
        (command_line::search::similarity_search::NAME, matches) => {
            command_line::search::similarity_search::action(matches)
        }
        (command_line::search::identity_search::NAME, matches) => {
            command_line::search::identity_search::action(matches)
        }
        (rest_api::NAME, matches) => rest_api::action(matches).await,
        (unknown, _) => panic!("🤨: {}", unknown),
    };

    matches?;

    Ok(())
}
