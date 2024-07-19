use crate::indexing::index_manager::IndexManager;
use crate::rest_api::api::{GetStructureSearchResponse, StructureResponseError};
use crate::search::scaffold_search::{scaffold_search, PARSED_SCAFFOLDS};
use crate::search::{
    get_smiles_and_extra_data, identity_search::identity_search, prepare_query_structure,
    StructureSearchHit,
};
use poem_openapi::payload::Json;

pub fn v1_index_search_identity(
    index_manager: &IndexManager,
    index: String,
    query_smiles: String,
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

    let query_attributes = prepare_query_structure(&query_smiles);

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
        Some(&PARSED_SCAFFOLDS)
    } else {
        None
    };

    let matching_scaffolds = match scaffolds {
        Some(scaffolds) => {
            let scaffold_matches = scaffold_search(&query_canon_taut, scaffolds);
            match scaffold_matches {
                Ok(scaffold_matches) => Some(scaffold_matches),
                Err(e) => {
                    return GetStructureSearchResponse::Err(Json(StructureResponseError {
                        error: e.to_string(),
                    }))
                }
            }
        }
        None => None,
    };

    let result = identity_search(
        &searcher,
        &query_canon_taut,
        &matching_scaffolds,
        fingerprint.0.as_bitslice(),
        &descriptors,
        extra_query,
    );

    match result {
        Ok(Some(result)) => {
            let schema = searcher.schema();
            let smiles_field = schema.get_field("smiles").unwrap();
            let extra_data_field = schema.get_field("extra_data").unwrap();

            let (smiles, extra_data) =
                get_smiles_and_extra_data(result, &searcher, smiles_field, extra_data_field)
                    .unwrap();

            GetStructureSearchResponse::Ok(Json(vec![StructureSearchHit {
                extra_data,
                smiles,
                score: 1.0,
                query: query_smiles,
                used_tautomers: false,
            }]))
        }
        Ok(None) => GetStructureSearchResponse::Ok(Json(Vec::new())),
        Err(e) => GetStructureSearchResponse::Err(Json(StructureResponseError {
            error: e.to_string(),
        })),
    }
}
