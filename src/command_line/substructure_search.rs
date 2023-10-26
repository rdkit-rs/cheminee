pub use super::prelude::*;
use crate::search::{
    aggregate_search_hits, compound_processing::*, prepare_query_structure,
    substructure_search::substructure_search,
};

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
            Arg::new("limit")
                .required(false)
                .long("limit")
                .short('l')
                .num_args(1),
        )
}

pub fn action(matches: &ArgMatches) -> eyre::Result<()> {
    let index_path = matches.get_one::<String>("index").unwrap();
    let smile = matches.get_one::<String>("smiles").unwrap();
    let result_limit = matches.get_one::<String>("limit");

    let (query_canon_taut, fingerprint, descriptors) = prepare_query_structure(smile)?;

    let index = open_index(index_path)?;
    let reader = index.reader()?;
    let searcher = reader.searcher();

    let result_limit = if let Some(result_limit) = result_limit {
        result_limit.parse::<usize>()?
    } else {
        usize::try_from(1000)?
    };

    let tautomer_limit = result_limit * 10;

    let mut results = substructure_search(
        &searcher,
        &query_canon_taut,
        fingerprint.0.as_bitslice(),
        &descriptors,
        result_limit,
    )?;

    let mut used_tautomers = false;
    let mut num_tauts_used = 0;
    if !results.is_empty() {
        num_tauts_used = 1;
    }

    if results.len() < result_limit {
        let tautomers = get_tautomers(&query_canon_taut);

        if tautomers.len() > 1 && tautomer_limit > 1 {
            for test_taut in tautomers {
                // don't reuse the canonical tautomer
                if test_taut.as_smile() == query_canon_taut.as_smile() {
                    continue;
                }

                let taut_attributes = get_cpd_properties(&test_taut);

                let taut_attributes = match taut_attributes {
                    Ok(taut_attributes) => taut_attributes,
                    Err(_) => continue,
                };

                let (taut_fingerprint, taut_descriptors) = taut_attributes;

                let taut_results = substructure_search(
                    &searcher,
                    &test_taut,
                    taut_fingerprint.0.as_bitslice(),
                    &taut_descriptors,
                    result_limit,
                );

                let taut_results = match taut_results {
                    Ok(taut_results) => taut_results,
                    Err(_) => continue,
                };

                results.extend(&taut_results);
                num_tauts_used += 1;

                if !used_tautomers {
                    used_tautomers = true;
                }

                if results.len() > result_limit || num_tauts_used == tautomer_limit {
                    break;
                }
            }
        }
    }

    let final_results = aggregate_search_hits(searcher, results, used_tautomers, smile)?;

    println!("{:#?}", final_results);

    Ok(())
}
