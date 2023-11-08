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
        .subcommand(command_line::fetch_pubchem::command())
        .subcommand(command_line::index_sdf::command())
        .subcommand(command_line::stream_pubchem_sdf::command())
        .subcommand(command_line::basic_search::command())
        .subcommand(rest_api::command())
        .subcommand(command_line::substructure_search::command())
        .subcommand(command_line::create_index::command())
        .subcommand(command_line::delete_index::command());

    let matches = app.get_matches();
    let matches = match matches.subcommand().unwrap() {
        (command_line::index_sdf::NAME, matches) => {
            let writes = command_line::index_sdf::action(matches)?;
            log::info!("wrote: {}", writes);
            Ok(())
        }
        (command_line::stream_pubchem_sdf::NAME, matches) => {
            command_line::stream_pubchem_sdf::action(matches)
        }
        (command_line::basic_search::NAME, matches) => command_line::basic_search::action(matches),
        (command_line::fetch_pubchem::NAME, matches) => {
            command_line::fetch_pubchem::action(matches).await
        }
        (rest_api::NAME, matches) => rest_api::action(matches).await,
        (command_line::substructure_search::NAME, matches) => {
            command_line::substructure_search::action(matches)
        }
        (command_line::create_index::NAME, matches) => command_line::create_index::action(matches),
        (command_line::delete_index::NAME, matches) => command_line::delete_index::action(matches),
        (unknown, _) => panic!("🤨: {}", unknown),
    };

    matches.unwrap();

    Ok(())
}
