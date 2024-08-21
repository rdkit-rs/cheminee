use crate::rest_api::api::{GetStructureSearchResponse, StructureResponseError};
use crate::search::compound_processing::{get_tautomers, standardize_smiles};
use crate::search::similarity_search::{get_best_similarity, similarity_search};
use crate::search::{aggregate_search_hits, validate_structure};
use poem_openapi::payload::Json;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use rdkit::Properties;
use std::cmp::min;
use std::collections::HashSet;
use tantivy::{DocAddress, Index};

pub fn v1_index_search_similarity(
    index: eyre::Result<Index>,
    smiles: String,
    result_limit: usize,
    tautomer_limit: usize,
    bin_limit: usize,
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
    let schema = searcher.schema();

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

    let tautomers = if tautomer_limit > 0 {
        let mut tauts = get_tautomers(&query_canon_taut);
        tauts.insert(0, query_canon_taut);
        tauts
    } else {
        vec![query_canon_taut]
    };

    let tautomer_limit = min(tautomers.len(), tautomer_limit + 1);
    let used_tautomers = tautomer_limit > 1;

    let mut results: HashSet<DocAddress> = HashSet::new();
    for taut in &tautomers[..tautomer_limit] {
        let taut_descriptors = Properties::new().compute_properties(taut);
        let taut_results = similarity_search(
            &searcher,
            &taut_descriptors,
            extra_query,
            result_limit,
            bin_limit,
            None,
        );
        if let Ok(taut_results) = taut_results {
            results.extend(taut_results);
        }
    }

    let taut_fingerprints = tautomers
        .iter()
        .map(|t| t.fingerprint())
        .collect::<Vec<_>>();

    let fingerprint_field = match schema.get_field("fingerprint") {
        Ok(fingerprint_field) => fingerprint_field,
        Err(e) => {
            return GetStructureSearchResponse::Err(Json(StructureResponseError {
                error: format!("Could not find \"fingerprint\" field: {e}"),
            }))
        }
    };

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
