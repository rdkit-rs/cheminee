use clap::*;

use cheminee::actions;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug");
    }
    tracing_subscriber::fmt::init();

    let app = Command::new("cheminee")
        .subcommand_required(true)
        .subcommand(actions::fetch_pubchem::command())
        .subcommand(actions::index_pubchem_sdf::command())
        .subcommand(actions::stream_pubchem_sdf::command())
        .subcommand(actions::search::command())
        .subcommand(actions::rest_api_server::command());

    let matches = app.get_matches();
    let matches = match matches.subcommand().unwrap() {
        (actions::index_pubchem_sdf::NAME, matches) => {
            let writes = actions::index_pubchem_sdf::action(matches)?;
            log::info!("wrote: {}", writes);
            Ok(())
        }
        (actions::stream_pubchem_sdf::NAME, matches) => {
            actions::stream_pubchem_sdf::action(matches)
        }
        (actions::search::NAME, matches) => actions::search::action(matches),
        (actions::fetch_pubchem::NAME, matches) => actions::fetch_pubchem::action(matches).await,
        (actions::rest_api_server::NAME, matches) => {
            actions::rest_api_server::action(matches).await
        }
        (unknown, _) => panic!("🤨: {}", unknown),
    };

    matches.unwrap();

    Ok(())
}
