pub use super::prelude::*;
use std::collections::HashMap;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use crate::analysis::compound_processing::*;

pub const NAME: &'static str = "substructure-search";

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
}

pub fn action(matches: &ArgMatches) -> eyre::Result<()> {
    let index_path = matches.get_one::<String>("index").unwrap();
    let smiles = matches.get_one::<String>("smiles").unwrap();

    let (canon_taut, fingerprint, descriptors) = process_cpd(smiles).unwrap();

    // let tautomers = get_tautomers(&canon_taut);


    // Iterate through integer fields and query for compounds that are "<" for each field
    // For now adding a placeholder query
    let query = "CrippenClogP: [0 TO 1]".to_string();
    let index = open_index(index_path)?;
    let schema = index.schema();

    let reader = index.reader()?;
    let searcher = reader.searcher();

    let query_parser = QueryParser::for_index(&index, vec![]);

    let _result = basic_search(&query_parser, &searcher, &schema, &query);

    Ok(())
}
