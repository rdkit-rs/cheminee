use crate::indexing::index_manager::IndexManager;
use crate::rest_api::api::{
    aggregate_search_hits, GetStructureSearchResponse, StructureResponseError,
};
use crate::search::compound_processing::{get_cpd_properties, get_tautomers};
use crate::search::prepare_query_structure;
use crate::search::substructure_search::substructure_search;
use poem_openapi::payload::Json;

pub fn v1_index_search_substructure(
    index_manager: &IndexManager,
    index: String,
    smile: String,
    limit: usize,
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

    let query_attributes = prepare_query_structure(&smile);

    let query_attributes = match query_attributes {
        Ok(query_attributes) => query_attributes,
        Err(e) => {
            return GetStructureSearchResponse::Err(Json(StructureResponseError {
                error: e.to_string(),
            }))
        }
    };

    let (query_canon_taut, fingerprint, descriptors) = query_attributes;

    let tantivy_result_limit = limit * 10;

    let results = substructure_search(
        &searcher,
        &query_canon_taut,
        fingerprint.0.as_bitslice(),
        &descriptors,
        tantivy_result_limit,
    );

    let mut results = match results {
        Ok(results) => results,
        Err(e) => {
            return GetStructureSearchResponse::Err(Json(StructureResponseError {
                error: e.to_string(),
            }))
        }
    };

    let mut tautomers_used = false;

    if results.len() < limit {
        let tautomers = get_tautomers(&query_canon_taut);

        let max_tauts = 10;

        for test_taut in tautomers.into_iter().take(max_tauts) {
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
                tantivy_result_limit,
            );

            let mut taut_results = match taut_results {
                Ok(taut_results) => taut_results,
                Err(_) => continue,
            };

            if taut_results.len() > 0 {
                tautomers_used = true;
            };

            results.append(&mut taut_results);

            if results.len() > limit {
                break;
            }
        }
    }

    let final_results = aggregate_search_hits(searcher, results, tautomers_used);

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
