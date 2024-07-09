use crate::command_line::prelude::*;
use crate::search::scaffold_search::{scaffold_search, PARSED_SCAFFOLDS};
use crate::search::{
    aggregate_search_hits, identity_search::identity_search, prepare_query_structure,
};
use std::collections::HashSet;
use tantivy::DocAddress;

pub const NAME: &str = "identity-search";

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
            Arg::new("extra_query")
                .required(false)
                .long("extra_query")
                .short('e')
                .num_args(1),
        )
        .arg(
            Arg::new("use_scaffolds")
                .required(false)
                .long("use_scaffolds")
                .short('u')
                .num_args(1),
        )
}

pub fn action(matches: &ArgMatches) -> eyre::Result<()> {
    let index_path = matches
        .get_one::<String>("index")
        .ok_or(eyre::eyre!("Failed to extract index path"))?;
    let smiles = matches
        .get_one::<String>("smiles")
        .ok_or(eyre::eyre!("Failed to extract SMILES"))?;
    let extra_query = matches.get_one::<String>("extra_query");
    let use_scaffolds = matches.get_one::<String>("use_scaffolds");

    let extra_query = if let Some(extra_query) = extra_query {
        extra_query.clone()
    } else {
        "".to_string()
    };

    // by default, we will use scaffold-based indexing
    let use_scaffolds = if let Some(use_scaffolds) = use_scaffolds {
        matches!(use_scaffolds.as_str(), "true")
    } else {
        true
    };

    let index = open_index(index_path)?;
    let reader = index.reader()?;
    let searcher = reader.searcher();

    let (query_canon_taut, fingerprint, descriptors) = prepare_query_structure(smiles)?;

    let scaffolds = if use_scaffolds {
        &PARSED_SCAFFOLDS
    } else {
        &Vec::new()
    };

    let matching_scaffolds = if !scaffolds.is_empty() {
        scaffold_search(&query_canon_taut, scaffolds)?
    } else {
        Vec::new()
    };

    let result = identity_search(
        &searcher,
        &query_canon_taut,
        &matching_scaffolds,
        fingerprint.0.as_bitslice(),
        &descriptors,
        &extra_query,
    )?;

    if let Some(result) = result {
        let mut results: HashSet<DocAddress> = HashSet::with_capacity(1);
        results.insert(result);

        let final_results = aggregate_search_hits(searcher, results, false, smiles)?;
        println!("{:#?}", final_results);
    } else {
        println!("No exact match result for {:?}", smiles);
    }

    Ok(())
}
