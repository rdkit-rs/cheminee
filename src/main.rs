use clap::*;

use cheminee::actions;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init();

    let app = Command::new("cheminee")
        .subcommand_required(true)
        .subcommand(actions::index_pubchem_sdf::command())
        .subcommand(actions::stream_pubchem_sdf::command());

    let matches = app.get_matches();
    let matches = match matches.subcommand().unwrap() {
        (actions::index_pubchem_sdf::NAME, matches) => actions::index_pubchem_sdf::action(matches),
        (actions::stream_pubchem_sdf::NAME, matches) => {
            actions::stream_pubchem_sdf::action(matches)
        }
        (unknown, _) => panic!("🤨: {}", unknown),
    };

    matches.unwrap();

    Ok(())
}
