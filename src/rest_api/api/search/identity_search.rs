use crate::indexing::index_manager::IndexManager;
use crate::rest_api::api::{GetStructureSearchResponse, StructureResponseError};
use crate::search::scaffold_search::{scaffold_search, PARSED_SCAFFOLDS};
use crate::search::{
    identity_search::identity_search,
    {aggregate_search_hits, prepare_query_structure},
};
use poem_openapi::payload::Json;
use std::collections::HashSet;
use tantivy::DocAddress;

pub fn v1_index_search_identity(
    index_manager: &IndexManager,
    index: String,
    smiles: String,
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

    let scaffolds = if use_scaffolds {
        &PARSED_SCAFFOLDS
    } else {
        &Vec::new()
    };

    let matching_scaffolds = if !scaffolds.is_empty() {
        let scaffold_matches = scaffold_search(&query_canon_taut, scaffolds);
        match scaffold_matches {
            Ok(scaffold_matches) => scaffold_matches,
            Err(e) => {
                return GetStructureSearchResponse::Err(Json(StructureResponseError {
                    error: e.to_string(),
                }))
            }
        }
    } else {
        Vec::new()
    };

    let result = identity_search(
        &searcher,
        &query_canon_taut,
        &matching_scaffolds,
        fingerprint.0.as_bitslice(),
        &descriptors,
        extra_query,
    );

    let mut results: HashSet<DocAddress> = HashSet::with_capacity(1);

    match result {
        Ok(Some(result)) => {
            results.insert(result);
        }
        Ok(None) => return GetStructureSearchResponse::Ok(Json(Vec::new())),
        Err(e) => {
            return GetStructureSearchResponse::Err(Json(StructureResponseError {
                error: e.to_string(),
            }))
        }
    };

    let final_results = aggregate_search_hits(searcher, results, false, &smiles);

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
