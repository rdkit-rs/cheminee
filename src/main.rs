use clap::*;

use cheminee::actions;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init();

    let app = Command::new("cheminee")
        .subcommand_required(true)
        .subcommand(actions::fetch_pubchem::command())
        .subcommand(actions::index_pubchem_sdf::command())
        .subcommand(actions::stream_pubchem_sdf::command())
        .subcommand(actions::search::command());

    let matches = app.get_matches();
    let matches = match matches.subcommand().unwrap() {
        (actions::index_pubchem_sdf::NAME, matches) => actions::index_pubchem_sdf::action(matches),
        (actions::stream_pubchem_sdf::NAME, matches) => {
            actions::stream_pubchem_sdf::action(matches)
        }
        (actions::search::NAME, matches) => actions::search::action(matches),
        (actions::fetch_pubchem::NAME, matches) => actions::fetch_pubchem::action(matches).await,
        (unknown, _) => panic!("🤨: {}", unknown),
    };

    matches.unwrap();

    Ok(())
}
