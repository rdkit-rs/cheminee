use crate::rest_api::api::{GetStructureSearchResponse, StructureResponseError};
use crate::search::compound_processing::standardize_smiles;
use crate::search::structure_search::structure_search;
use crate::search::{
    compound_processing::get_tautomers, sort_results, validate_structure, StructureSearchHit,
};
use poem_openapi::payload::Json;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use std::cmp::min;
use tantivy::Index;

pub fn v1_index_search_structure(
    index: eyre::Result<Index>,
    query_smiles: String,
    use_chirality: bool,
    method: &str,
    result_limit: usize,
    tautomer_limit: usize,
    extra_query: &str,
    use_scaffolds: bool,
) -> GetStructureSearchResponse {
    let index = match index {
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

    let problems_result = validate_structure(&query_smiles);

    let problems_exist = match problems_result {
        Ok(problems) => !problems.is_empty(),
        Err(_) => true,
    };

    if problems_exist {
        return GetStructureSearchResponse::Err(Json(StructureResponseError {
            error: "Failed structure validation".to_string(),
        }));
    };

    let query_canon_taut = match standardize_smiles(&query_smiles, false) {
        Ok(romol) => romol,
        Err(e) => {
            return GetStructureSearchResponse::Err(Json(StructureResponseError {
                error: format!("Failed structure standardization: {e}"),
            }));
        }
    };

    let results = structure_search(
        &searcher,
        &query_canon_taut,
        method,
        use_scaffolds,
        result_limit,
        use_chirality,
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

    if before_tauts_result_count < result_limit && tautomer_limit > 0 {
        let mut tautomers = get_tautomers(&query_canon_taut);

        let tautomer_limit = min(tautomers.len(), tautomer_limit);

        if !tautomers.is_empty() {
            let tautomer_results = tautomers[..tautomer_limit]
                .into_par_iter()
                .filter_map(|taut| {
                    structure_search(
                        &searcher,
                        taut,
                        method,
                        use_scaffolds,
                        result_limit,
                        use_chirality,
                        extra_query,
                    )
                    .ok()
                })
                .collect::<Vec<_>>();

            for results_set in tautomer_results {
                if results.len() < result_limit {
                    results.extend(results_set);
                }
            }

            if results.len() > before_tauts_result_count {
                used_tautomers = true;
            }
        }
    }

    let mut data_results = results.into_iter().collect::<Vec<_>>();

    let final_results = sort_results(&mut data_results)
        .into_iter()
        .map(|(smiles, extra_data)| StructureSearchHit {
            extra_data,
            smiles,
            score: 1.0,
            query: query_smiles.clone(),
            used_tautomers,
        })
        .collect::<Vec<_>>();

    if final_results.len() > result_limit {
        GetStructureSearchResponse::Ok(Json(final_results[..result_limit].into()))
    } else {
        GetStructureSearchResponse::Ok(Json(final_results))
    }
}
