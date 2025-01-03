use crate::rest_api::api::{GetStructureSearchResponse, StructureResponseError};
use crate::search::scaffold_search::{scaffold_search, PARSED_SCAFFOLDS};
use crate::search::{
    identity_search::identity_search, prepare_query_structure, sort_results, StructureSearchHit,
};
use poem_openapi::payload::Json;
use tantivy::Index;

pub fn v1_index_search_identity(
    index: eyre::Result<Index>,
    query_smiles: String,
    use_chirality: bool,
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

    let query_attributes = prepare_query_structure(&query_smiles);

    let query_attributes = match query_attributes {
        Ok(query_attributes) => query_attributes,
        Err(e) => {
            return GetStructureSearchResponse::Err(Json(StructureResponseError {
                error: e.to_string(),
            }))
        }
    };

    let (query_canon_taut, pattern_fingerprint, descriptors) = query_attributes;

    let matching_scaffolds = if use_scaffolds {
        scaffold_search(&pattern_fingerprint.0, &query_canon_taut, &PARSED_SCAFFOLDS).ok()
    } else {
        None
    };

    let data_results = identity_search(
        &searcher,
        &query_canon_taut,
        &matching_scaffolds,
        pattern_fingerprint.0.as_bitslice(),
        &descriptors,
        use_chirality,
        extra_query,
    );

    let final_results = match data_results {
        Ok(mut data_results) => sort_results(&mut data_results)
            .into_iter()
            .map(|(smiles, extra_data)| StructureSearchHit {
                extra_data,
                smiles,
                score: 1.0,
                query: query_smiles.clone(),
                used_tautomers: false,
            })
            .collect::<Vec<_>>(),
        Err(e) => {
            return GetStructureSearchResponse::Err(Json(StructureResponseError {
                error: e.to_string(),
            }))
        }
    };

    GetStructureSearchResponse::Ok(Json(final_results))
}
