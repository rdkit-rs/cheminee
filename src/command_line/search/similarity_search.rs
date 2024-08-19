use crate::command_line::prelude::*;
use crate::search::compound_processing::{get_tautomers, standardize_smiles};
use crate::search::similarity_search::{get_best_similarity, similarity_search};
use crate::search::{aggregate_search_hits, validate_structure};
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelIterator;
use rdkit::Properties;
use std::cmp::min;
use std::collections::HashSet;
use tantivy::DocAddress;

pub const NAME: &str = "similarity-search";

pub fn command() -> Command {
    Command::new(crate::command_line::search::similarity_search::NAME)
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
    let smiles = matches
        .get_one::<String>("smiles")
        .ok_or(eyre::eyre!("Failed to extract SMILES"))?;
    let result_limit = matches.get_one::<String>("result-limit");
    let tautomer_limit = matches.get_one::<String>("tautomer-limit");
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

    let extra_query = if let Some(extra_query) = extra_query {
        extra_query.clone()
    } else {
        "".to_string()
    };

    let index = open_index(index_path)?;
    let reader = index.reader()?;
    let searcher = reader.searcher();
    let schema = searcher.schema();

    let problems = validate_structure(smiles)?;
    if !problems.is_empty() {
        return Err(eyre::eyre!("Failed structure validation"));
    };

    let query_canon_taut = standardize_smiles(smiles, false)?;

    let mut tautomers = if tautomer_limit > 0 {
        let mut tauts = get_tautomers(&query_canon_taut);
        tauts.insert(0, query_canon_taut);
        tauts
    } else {
        vec![query_canon_taut]
    };

    let tautomer_limit = min(tautomers.len(), tautomer_limit + 1);

    let used_tautomers = tautomer_limit > 1;

    let mut results: HashSet<DocAddress> = HashSet::new();
    let tautomer_results = &tautomers[..tautomer_limit]
        .into_par_iter()
        .filter_map(|taut| {
            let taut_descriptors = Properties::new().compute_properties(taut);
            similarity_search(
                &searcher,
                &taut_descriptors,
                10 * result_limit,
                &extra_query,
            )
            .ok()
        })
        .collect::<Vec<_>>();

    for results_set in tautomer_results {
        results.extend(results_set);
    }

    let taut_fingerprints = tautomers
        .iter()
        .map(|t| t.fingerprint())
        .collect::<Vec<_>>();

    let fingerprint_field = schema.get_field("fingerprint")?;

    let mut results = results
        .into_par_iter()
        .map(|docaddr| {
            let sim =
                get_best_similarity(&searcher, &docaddr, fingerprint_field, &taut_fingerprints);

            if let Ok(sim) = sim {
                (docaddr, sim)
            } else {
                (docaddr, 0.0)
            }
        })
        .collect::<Vec<_>>();

    // Sort by descending similarity
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let results = if results.len() > result_limit {
        results[..result_limit].to_vec()
    } else {
        results
    };

    let final_results = aggregate_search_hits(searcher, results, used_tautomers, smiles)?;

    println!("{:#?}", final_results);

    Ok(())
}
