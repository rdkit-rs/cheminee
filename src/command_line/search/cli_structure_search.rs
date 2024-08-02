use crate::command_line::prelude::*;
use crate::search::structure_search::structure_search;
use crate::search::{aggregate_search_hits, compound_processing::*, validate_structure};
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelIterator;

pub fn cli_structure_search(method: &str, matches: &ArgMatches) -> eyre::Result<()> {
    let index_path = matches
        .get_one::<String>("index")
        .ok_or(eyre::eyre!("Failed to extract index path"))?;
    let smiles = matches
        .get_one::<String>("smiles")
        .ok_or(eyre::eyre!("Failed to extract SMILES"))?;
    let result_limit = matches.get_one::<String>("result-limit");
    let tautomer_limit = matches.get_one::<String>("tautomer-limit");
    let extra_query = matches.get_one::<String>("extra-query");
    let use_scaffolds = matches.get_one::<String>("use-scaffolds");

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

    // by default, we will use scaffold-based indexing
    let use_scaffolds = if let Some(use_scaffolds) = use_scaffolds {
        matches!(use_scaffolds.as_str(), "true")
    } else {
        true
    };

    let index = open_index(index_path)?;
    let reader = index.reader()?;
    let searcher = reader.searcher();

    let problems = validate_structure(smiles)?;
    if !problems.is_empty() {
        return Err(eyre::eyre!("Failed structure validation"));
    };

    let query_canon_taut = standardize_smiles(smiles, false)?;

    let mut results = structure_search(
        &searcher,
        &query_canon_taut,
        method,
        use_scaffolds,
        result_limit,
        &extra_query,
    )?;

    let mut used_tautomers = false;
    let before_tauts_result_count = results.len();

    if before_tauts_result_count < result_limit {
        let tautomers = get_tautomers(&query_canon_taut);

        if !tautomers.is_empty() && tautomer_limit > 0 {
            let tautomer_results = tautomers
                .into_par_iter()
                .filter_map(|taut| {
                    structure_search(
                        &searcher,
                        &taut,
                        method,
                        use_scaffolds,
                        result_limit,
                        &extra_query,
                    )
                    .ok()
                })
                .collect::<Vec<_>>();

            for results_set in tautomer_results {
                results.extend(&results_set);
            }

            if results.len() > before_tauts_result_count {
                used_tautomers = true;
            }
        }
    }

    let final_results = aggregate_search_hits(searcher, results, used_tautomers, smiles)?;

    println!("{:#?}", final_results);

    Ok(())
}
