use crate::command_line::prelude::*;
use crate::search::superstructure_search::run_superstructure_search;
use crate::search::{aggregate_search_hits, compound_processing::*, validate_structure};
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;

pub const NAME: &str = "superstructure-search";

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
                .help("An extra query (e.g. \"exactmw:[50 TO 100]\") may be helpful in case you want to further restrict the kinds of superstructure matches that are returned")
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

    let mut results = run_superstructure_search(
        &searcher,
        &query_canon_taut,
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
                    run_superstructure_search(
                        &searcher,
                        &taut,
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
