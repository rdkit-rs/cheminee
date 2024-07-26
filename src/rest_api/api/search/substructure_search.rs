use crate::indexing::index_manager::IndexManager;
use crate::rest_api::api::{GetStructureSearchResponse, StructureResponseError};
use crate::search::compound_processing::standardize_smiles;
use crate::search::substructure_search::run_substructure_search;
use crate::search::{
    aggregate_search_hits, compound_processing::get_tautomers, validate_structure,
};
use poem_openapi::payload::Json;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;

pub fn v1_index_search_substructure(
    index_manager: &IndexManager,
    index: String,
    smiles: String,
    result_limit: usize,
    tautomer_limit: usize,
    extra_query: &str,
    use_scaffolds: bool,
) -> GetStructureSearchResponse {
    let index = match index_manager.open(&index) {
        Ok(index) => index,
        Err(e) => {
            return GetStructureSearchResponse::Err(Json(StructureResponseError {
                error: e.to_string(),
            }))
        }
    };

    let reader = index.reader();
    let reader = match reader {
        Ok(reader) => reader,
        Err(e) => {
            return GetStructureSearchResponse::Err(Json(StructureResponseError {
                error: e.to_string(),
            }))
        }
    };

    let searcher = reader.searcher();

    let problems_result = validate_structure(&smiles);

    let problems_exist = match problems_result {
        Ok(problems) => !problems.is_empty(),
        Err(_) => true,
    };

    if problems_exist {
        return GetStructureSearchResponse::Err(Json(StructureResponseError {
            error: "Failed structure validation".to_string(),
        }));
    };

    let query_canon_taut = match standardize_smiles(&smiles, false) {
        Ok(romol) => romol,
        Err(e) => {
            return GetStructureSearchResponse::Err(Json(StructureResponseError {
                error: format!("Failed structure standardization: {e}"),
            }));
        }
    };

    let results = run_substructure_search(
        &searcher,
        &query_canon_taut,
        use_scaffolds,
        result_limit,
        extra_query,
    );

    let mut results = match results {
        Ok(results) => results,
        Err(e) => {
            return GetStructureSearchResponse::Err(Json(StructureResponseError {
                error: e.to_string(),
            }))
        }
    };

    let mut used_tautomers = false;
    let before_tauts_result_count = results.len();

    if before_tauts_result_count < result_limit {
        let tautomers = get_tautomers(&query_canon_taut);

        if !tautomers.is_empty() && tautomer_limit > 0 {
            let tautomer_results = tautomers
                .into_par_iter()
                .filter_map(|taut| {
                    run_substructure_search(
                        &searcher,
                        &taut,
                        use_scaffolds,
                        result_limit,
                        extra_query,
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

    let final_results = aggregate_search_hits(searcher, results, used_tautomers, &smiles);

    let final_results = match final_results {
        Ok(final_results) => final_results,
        Err(e) => {
            return GetStructureSearchResponse::Err(Json(StructureResponseError {
                error: e.to_string(),
            }))
        }
    };

    GetStructureSearchResponse::Ok(Json(final_results))
}
