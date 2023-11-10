pub use super::super::prelude::*;
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
            Arg::new("result_limit")
                .required(false)
                .long("result_limit")
                .short('r')
                .num_args(1),
        )
        .arg(
            Arg::new("tautomer_limit")
                .required(false)
                .long("tautomer_limit")
                .short('t')
                .num_args(1),
        )
        .arg(
            Arg::new("extra_query")
                .required(false)
                .long("extra_query")
                .short('e')
                .num_args(1),
        )
}

pub fn action(matches: &ArgMatches) -> eyre::Result<()> {
    let index_path = matches.get_one::<String>("index").unwrap();
    let smile = matches.get_one::<String>("smiles").unwrap();
    let result_limit = matches.get_one::<String>("result_limit");
    let tautomer_limit = matches.get_one::<String>("tautomer_limit");
    let extra_query = matches.get_one::<String>("extra_query");

    let result_limit = if let Some(result_limit) = result_limit {
        result_limit.parse::<usize>()?
    } else {
        usize::try_from(1000)?
    };

    let tautomer_limit = if let Some(tautomer_limit) = tautomer_limit {
        tautomer_limit.parse::<usize>()?
    } else {
        usize::try_from(10)?
    };

    let extra_query = if let Some(extra_query) = extra_query {
        extra_query.clone()
    } else {
        "".to_string()
    };

    let index = open_index(index_path)?;
    let reader = index.reader()?;
    let searcher = reader.searcher();

    let (query_canon_taut, fingerprint, descriptors) = prepare_query_structure(smile)?;

    let mut results = substructure_search(
        &searcher,
        &query_canon_taut,
        fingerprint.0.as_bitslice(),
        &descriptors,
        result_limit,
        &extra_query,
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
                    &extra_query,
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
