use crate::command_line::prelude::*;
use crate::search::similarity_search::{get_best_similarity, similarity_search};
use crate::search::{compound_processing::*, validate_structure, StructureSearchHit};
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelIterator;
use std::cmp::min;
use std::collections::HashSet;
use tantivy::DocAddress;

pub const NAME: &str = "similarity-search";

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
                .help("Set to 0 by default; Cheminee will use the specified number of tautomers to boost search results")
                .num_args(1),
        )
        .arg(
            Arg::new("search-percent-limit")
                .required(false)
                .long("search-percent-limit")
                .short('p')
                .help("Set to 0.1% by default; Cheminee will aim to search this percentage of the database for similar compounds")
                .num_args(1),
        )
        .arg(
            Arg::new("extra-query")
                .required(false)
                .long("extra-query")
                .short('e')
                .help("An extra query (e.g. \"exactmw:[50 TO 100]\") may be helpful in case you want to further restrict the kinds of similarity matches that are returned")
                .num_args(1),
        )
}

pub fn action(matches: &ArgMatches) -> eyre::Result<()> {
    let index_path = matches
        .get_one::<String>("index")
        .ok_or(eyre::eyre!("Failed to extract index path"))?;
    let query_smiles = matches
        .get_one::<String>("smiles")
        .ok_or(eyre::eyre!("Failed to extract SMILES"))?;
    let result_limit = matches.get_one::<String>("result-limit");
    let tautomer_limit = matches.get_one::<String>("tautomer-limit");
    let search_percent_limit = matches.get_one::<String>("search-percent-limit");
    let extra_query = matches.get_one::<String>("extra-query");

    let result_limit = if let Some(result_limit) = result_limit {
        result_limit.parse::<usize>()?
    } else {
        usize::try_from(1000)?
    };

    let tautomer_limit = if let Some(tautomer_limit) = tautomer_limit {
        tautomer_limit.parse::<usize>()?
    } else {
        usize::try_from(0)?
    };

    let search_percent_limit = if let Some(search_percent_limit) = search_percent_limit {
        search_percent_limit.parse::<f32>()?
    } else {
        0.1
    };

    let extra_query = if let Some(extra_query) = extra_query {
        extra_query.clone()
    } else {
        "".to_string()
    };

    let index = open_index(index_path)?;
    let reader = index.reader()?;
    let searcher = reader.searcher();

    let problems = validate_structure(query_smiles)?;
    if !problems.is_empty() {
        return Err(eyre::eyre!("Failed structure validation"));
    };

    let query_canon_taut = standardize_smiles(query_smiles, false)?;

    let tautomers = if tautomer_limit > 0 {
        let mut tauts = get_tautomers(&query_canon_taut);
        tauts.insert(0, query_canon_taut);
        tauts
    } else {
        vec![query_canon_taut]
    };

    let tautomer_limit = min(tautomers.len(), tautomer_limit + 1);

    let used_tautomers = tautomer_limit > 1;

    let taut_morgan_fingerprints = tautomers[..tautomer_limit]
        .iter()
        .map(|m| m.morgan_fingerprint().0)
        .collect::<Vec<_>>();

    let mut results: HashSet<DocAddress> = HashSet::new();
    for taut_fp in &taut_morgan_fingerprints {
        let taut_results =
            similarity_search(&searcher, taut_fp, &extra_query, search_percent_limit);
        if let Ok(taut_results) = taut_results {
            results.extend(taut_results);
        } else {
            log::warn!("Encountered a failed search");
        }
    }

    let schema = searcher.schema();
    let smiles_field = schema.get_field("smiles")?;
    let morgan_fingerprint_field = schema.get_field("morgan_fingerprint")?;
    let extra_data_field = schema.get_field("extra_data")?;

    let mut final_results = results
        .into_par_iter()
        .filter_map(|docaddr| {
            let result = get_best_similarity(
                &searcher,
                &docaddr,
                smiles_field,
                morgan_fingerprint_field,
                extra_data_field,
                &taut_morgan_fingerprints,
            );

            match result {
                Ok(result) => Some(StructureSearchHit {
                    smiles: result.0,
                    extra_data: result.1,
                    score: result.2,
                    query: query_smiles.into(),
                    used_tautomers,
                }),
                Err(e) => {
                    log::warn!("Encountered exception in Tanimoto calculation: {e}");
                    None
                }
            }
        })
        .collect::<Vec<_>>();

    final_results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if final_results.len() > result_limit {
        log::info!("{:#?}", &final_results[..result_limit]);
    } else {
        log::info!("{:#?}", final_results)
    }

    Ok(())
}
