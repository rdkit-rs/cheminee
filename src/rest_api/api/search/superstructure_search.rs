use crate::indexing::index_manager::IndexManager;
use crate::rest_api::api::{GetStructureSearchResponse, StructureResponseError};
use crate::search::scaffold_search::{scaffold_search, PARSED_SCAFFOLDS};
use crate::search::{
    compound_processing::{get_cpd_properties, get_tautomers},
    superstructure_search::superstructure_search,
    {aggregate_search_hits, prepare_query_structure},
};
use poem_openapi::payload::Json;

pub fn v1_index_search_superstructure(
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

    let query_attributes = prepare_query_structure(&smiles);

    let query_attributes = match query_attributes {
        Ok(query_attributes) => query_attributes,
        Err(e) => {
            return GetStructureSearchResponse::Err(Json(StructureResponseError {
                error: e.to_string(),
            }))
        }
    };

    let (query_canon_taut, fingerprint, descriptors) = query_attributes;

    let matching_scaffolds = if use_scaffolds {
        scaffold_search(&query_canon_taut, &PARSED_SCAFFOLDS).ok()
    } else {
        None
    };

    let results = superstructure_search(
        &searcher,
        &query_canon_taut,
        &matching_scaffolds,
        fingerprint.0.as_bitslice(),
        &descriptors,
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
    let mut num_tauts_used = 0;
    if !results.is_empty() {
        num_tauts_used = 1;
    }

    if results.len() < result_limit {
        let tautomers = get_tautomers(&query_canon_taut);

        if tautomers.len() > 1 && tautomer_limit > 1 {
            for test_taut in tautomers {
                // don't reuse the canonical tautomer
                if test_taut.as_smiles() == query_canon_taut.as_smiles() {
                    continue;
                }

                let taut_attributes = get_cpd_properties(&test_taut);

                let taut_attributes = match taut_attributes {
                    Ok(taut_attributes) => taut_attributes,
                    Err(_) => continue,
                };

                let (taut_fingerprint, taut_descriptors) = taut_attributes;

                let matching_scaffolds = if use_scaffolds {
                    scaffold_search(&test_taut, &PARSED_SCAFFOLDS).ok()
                } else {
                    None
                };

                let taut_results = superstructure_search(
                    &searcher,
                    &test_taut,
                    &matching_scaffolds,
                    taut_fingerprint.0.as_bitslice(),
                    &taut_descriptors,
                    result_limit,
                    extra_query,
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
