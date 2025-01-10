use crate::indexing::index_manager::IndexManager;
use crate::rest_api::api::{
    BulkRequest, BulkRequestDoc, DeleteIndexBulkResponseError, DeleteIndexBulkResponseOk,
    DeleteIndexBulkResponseOkStatus, DeleteIndexesBulkDeleteResponse,
};
use crate::search::compound_processing::process_cpd;
use crate::search::scaffold_search::{scaffold_search, PARSED_SCAFFOLDS};
use poem_openapi::payload::Json;
use tantivy::query::{Query, QueryParser};

pub async fn v1_delete_index_bulk(
    index_manager: &IndexManager,
    index: String,
    bulk_request: BulkRequest,
) -> DeleteIndexesBulkDeleteResponse {
    let index = match index_manager.open(&index) {
        Ok(index) => index,
        Err(e) => {
            return DeleteIndexesBulkDeleteResponse::Err(Json(DeleteIndexBulkResponseError {
                error: e.to_string(),
            }))
        }
    };

    let mut deleter = match index.writer::<tantivy::TantivyDocument>(50 * 1024 * 1024) {
        Ok(deleter) => deleter,
        Err(e) => {
            return DeleteIndexesBulkDeleteResponse::Err(Json(DeleteIndexBulkResponseError {
                error: e.to_string(),
            }))
        }
    };

    let query_parser = QueryParser::for_index(&index, vec![]);

    let mut document_delete_statuses = Vec::with_capacity(bulk_request.docs.len());
    for bulk_request_doc in bulk_request.docs {
        let parsed_query = bulk_request_doc_to_query(&bulk_request_doc, &query_parser);

        let status = match parsed_query {
            Ok(parsed_query) => {
                let delete_operation = deleter.delete_query(parsed_query);

                match delete_operation {
                    Ok(opstamp) => DeleteIndexBulkResponseOkStatus {
                        opcode: Some(opstamp),
                        error: None,
                    },
                    Err(e) => DeleteIndexBulkResponseOkStatus {
                        opcode: None,
                        error: Some(e.to_string()),
                    },
                }
            }
            Err(e) => DeleteIndexBulkResponseOkStatus {
                opcode: None,
                error: Some(e.to_string()),
            },
        };
        document_delete_statuses.push(status);
    }

    match deleter.commit() {
        Ok(_) => (),
        Err(e) => {
            return DeleteIndexesBulkDeleteResponse::Err(Json(DeleteIndexBulkResponseError {
                error: e.to_string(),
            }))
        }
    }

    DeleteIndexesBulkDeleteResponse::Ok(Json(DeleteIndexBulkResponseOk {
        statuses: document_delete_statuses,
    }))
}

fn bulk_request_doc_to_query(
    bulk_request_doc: &BulkRequestDoc,
    query_parser: &QueryParser,
) -> eyre::Result<Box<dyn Query>> {
    let (canon_taut, pattern_fingerprint, descriptors) =
        process_cpd(&bulk_request_doc.smiles, false)?;

    let canon_smiles = canon_taut.as_smiles();
    let matching_scaffolds =
        scaffold_search(&pattern_fingerprint.0, &canon_taut, &PARSED_SCAFFOLDS);
    let matching_scaffolds = match matching_scaffolds {
        Ok(matching_scaffolds) => Some(matching_scaffolds),
        Err(_) => None,
    };

    let raw_query =
        crate::search::identity_search::build_identity_query(&descriptors, "", &matching_scaffolds);
    let query = format!("{raw_query} AND smiles:\"{canon_smiles}\"");
    let parsed_query = query_parser.parse_query(&query)?;
    Ok(parsed_query)
}
