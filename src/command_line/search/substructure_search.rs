use crate::command_line::prelude::*;
use crate::command_line::search::cli_structure_search::cli_structure_search;

pub const NAME: &str = "substructure-search";

pub fn command() -> Command {
    Command::new(NAME)
        .arg(
            Arg::new("index")
                .required(true)
                .long("index")
                .short('i')
                .num_args(1),
        )
        .arg(
            Arg::new("smiles")
                .required(true)
                .long("smiles")
                .short('s')
                .num_args(1),
        )
        .arg(
            Arg::new("result-limit")
                .required(false)
                .long("result-limit")
                .short('r')
                .num_args(1),
        )
        .arg(
            Arg::new("tautomer-limit")
                .required(false)
                .long("tautomer-limit")
                .short('t')
                .num_args(1),
        )
        .arg(
            Arg::new("extra-query")
                .required(false)
                .long("extra-query")
                .short('e')
                .help("An extra query (e.g. \"exactmw:[50 TO 100]\") may be helpful in case you want to further restrict the kinds of substructure matches that are returned")
                .num_args(1),
        )
        .arg(
            Arg::new("use-scaffolds")
                .required(false)
                .long("use-scaffolds")
                .short('u')
                .help("By default scaffolds are computed for the smiles input to enable accelerated searching")
                .num_args(1),
        )
}

pub fn action(matches: &ArgMatches) -> eyre::Result<()> {
    cli_structure_search("substructure", matches)
}
