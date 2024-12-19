use crate::rest_api::api::{GetStructureSearchResponse, StructureResponseError};
use crate::search::compound_processing::{get_tautomers, standardize_smiles};
use crate::search::similarity_search::neighbor_search;
use crate::search::{similarity_search::similarity_search, validate_structure};
use poem_openapi::payload::Json;
use std::cmp::min;
use std::collections::HashSet;
use tantivy::Index;

pub fn v1_index_search_similarity(
    index: eyre::Result<Index>,
    query_smiles: String,
    result_limit: usize,
    tautomer_limit: usize,
    search_percent_limit: f32,
    tanimoto_minimum: f32,
    extra_query: &str,
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

    let tautomers = if tautomer_limit > 0 {
        let mut tauts = get_tautomers(&query_canon_taut);
        tauts.insert(0, query_canon_taut);
        tauts
    } else {
        vec![query_canon_taut]
    };

    let tautomer_limit = min(tautomers.len(), tautomer_limit + 1);

    let taut_morgan_fingerprints = tautomers[..tautomer_limit]
        .iter()
        .map(|m| m.morgan_fingerprint().0)
        .collect::<Vec<_>>();

    let results = neighbor_search(&searcher, &taut_morgan_fingerprints, extra_query, search_percent_limit).unwrap_or_else(|e| {
        log::warn!("Encountered a failed search: {e}");
        HashSet::new()
    });

    let final_results = match similarity_search(
        &searcher,
        results,
        &taut_morgan_fingerprints,
        tanimoto_minimum,
        &query_smiles,
    ) {
        Ok(final_results) => final_results,
        Err(e) => {
            return GetStructureSearchResponse::Err(Json(StructureResponseError {
                error: e.to_string(),
            }))
        }
    };

    if final_results.len() > result_limit {
        GetStructureSearchResponse::Ok(Json(final_results[..result_limit].into()))
    } else {
        GetStructureSearchResponse::Ok(Json(final_results))
    }
}
