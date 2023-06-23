pub use super::prelude::*;
use std::collections::HashMap;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use crate::analysis::compound_processing::*;
use crate::search::substructure_search::substructure_search;
use crate::search::validate_structure;

pub const NAME: &'static str = "substructure-search";

pub fn command() -> Command {
    Command::new(NAME)
        .arg(
            Arg::new("index")
                .required(true)
                .long("index")
                .short('i')
                .num_args(1)
        )
        .arg(
            Arg::new("smiles")
                .required(true)
                .long("smiles")
                .short('s')
                .num_args(1)
        )
        .arg(
            Arg::new("limit")
                .required(false)
                .long("limit")
                .short('l')
                .num_args(1)
        )

}

pub fn action(matches: &ArgMatches) -> eyre::Result<()> {
    let index_path = matches.get_one::<String>("index").unwrap();
    let smiles = matches.get_one::<String>("smiles").unwrap();
    let limit = matches.get_one::<String>("limit");

    // Validate structure
    let problems = validate_structure(smiles);
    if problems.len() > 0 {
        panic!("Need to do something here to either correct the smiles query or return an error message");
    }

    // Need to process cpd ahead of time as we may need to iterate through tautomers later
    let (canon_taut, fingerprint, descriptors) = process_cpd(smiles).unwrap();

    // let tautomers = get_tautomers(&canon_taut);

    let index = open_index(index_path)?;
    let reader = index.reader()?;
    let searcher = reader.searcher();


    let limit = if let Some(limit) = limit {
        limit.parse::<usize>()?
    } else {
        usize::try_from(1000).unwrap()
    };

    let _result = substructure_search(&searcher, &canon_taut, fingerprint.0.as_bitslice(), &descriptors, limit);

    Ok(())
}
