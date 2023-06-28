use clap::*;

use cheminee::command_line;
use cheminee::rest_api;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info,poem=debug");
    }
    tracing_subscriber::fmt::init();

    let app = Command::new("cheminee")
        .subcommand_required(true)
        .subcommand(command_line::fetch_pubchem::command())
        .subcommand(command_line::index_pubchem_sdf::command())
        .subcommand(command_line::stream_pubchem_sdf::command())
        .subcommand(command_line::basic_search::command())
        .subcommand(rest_api::command())
        .subcommand(command_line::substructure_search::command());

    let matches = app.get_matches();
    let matches = match matches.subcommand().unwrap() {
        (command_line::index_pubchem_sdf::NAME, matches) => {
            let writes = command_line::index_pubchem_sdf::action(matches)?;
            log::info!("wrote: {}", writes);
            Ok(())
        }
        (command_line::stream_pubchem_sdf::NAME, matches) => {
            command_line::stream_pubchem_sdf::action(matches)
        }
        (command_line::basic_search::NAME, matches) => command_line::basic_search::action(matches),
        (command_line::fetch_pubchem::NAME, matches) => command_line::fetch_pubchem::action(matches).await,
        (rest_api::NAME, matches) => {
            rest_api::action(matches).await
        }
        (command_line::substructure_search::NAME, matches) => {
            command_line::substructure_search::action(matches)
        }
        (unknown, _) => panic!("🤨: {}", unknown),
    };

    matches.unwrap();

    Ok(())
}
