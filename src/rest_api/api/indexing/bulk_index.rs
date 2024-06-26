use crate::indexing::{index_manager::IndexManager, KNOWN_DESCRIPTORS};
use crate::rest_api::api::{
    BulkRequest, BulkRequestDoc, PostIndexBulkResponseError, PostIndexBulkResponseOk,
    PostIndexBulkResponseOkStatus, PostIndexesBulkIndexResponse,
};
use crate::search::compound_processing::process_cpd;
use poem_openapi::payload::Json;
use rayon::prelude::*;
use serde_json::{Map, Value};
use std::collections::HashMap;
use tantivy::schema::Field;

pub async fn v1_post_index_bulk(
    index_manager: &IndexManager,
    index: String,
    bulk_request: BulkRequest,
) -> PostIndexesBulkIndexResponse {
    let index = match index_manager.open(&index) {
        Ok(index) => index,
        Err(e) => {
            return PostIndexesBulkIndexResponse::Err(Json(PostIndexBulkResponseError {
                error: e.to_string(),
            }))
        }
    };

    let mut writer = match index.writer(8 * 1024 * 1024) {
        Ok(writer) => writer,
        Err(e) => {
            return PostIndexesBulkIndexResponse::Err(Json(PostIndexBulkResponseError {
                error: e.to_string(),
            }))
        }
    };

    let schema = index.schema();

    let smiles_field = schema.get_field("smiles").unwrap();
    let fingerprint_field = schema.get_field("fingerprint").unwrap();
    let extra_data_field = schema.get_field("extra_data").unwrap();

    let descriptors_fields = KNOWN_DESCRIPTORS
        .iter()
        .map(|kd| (*kd, schema.get_field(kd).unwrap()))
        .collect::<HashMap<&str, Field>>();

    let tantivy_docs_conversion_operation = tokio::task::spawn_blocking(move || {
        bulk_request
            .docs
            .into_par_iter()
            .map(|doc| {
                bulk_request_doc_to_tantivy_doc(
                    doc,
                    smiles_field,
                    fingerprint_field,
                    &descriptors_fields,
                    extra_data_field,
                )
            })
            .collect::<Vec<_>>()
    })
    .await;

    let tantivy_docs = match tantivy_docs_conversion_operation {
        Ok(docs) => docs,
        Err(e) => {
            return PostIndexesBulkIndexResponse::Err(Json(PostIndexBulkResponseError {
                error: e.to_string(),
            }))
        }
    };

    let mut document_insert_statuses = Vec::with_capacity(tantivy_docs.len());

    for doc_conversion_result in tantivy_docs {
        let tantivy_doc = match doc_conversion_result {
            Ok(doc) => doc,
            Err(e) => {
                document_insert_statuses.push(PostIndexBulkResponseOkStatus {
                    opcode: None,
                    error: Some(e.to_string()),
                });
                continue;
            }
        };

        let write_operation = writer.add_document(tantivy_doc);

        let status = match write_operation {
            Ok(opstamp) => PostIndexBulkResponseOkStatus {
                opcode: Some(opstamp),
                error: None,
            },
            Err(e) => PostIndexBulkResponseOkStatus {
                opcode: None,
                error: Some(e.to_string()),
            },
        };
        document_insert_statuses.push(status);
    }

    match writer.commit() {
        Ok(_) => (),
        Err(e) => {
            return PostIndexesBulkIndexResponse::Err(Json(PostIndexBulkResponseError {
                error: e.to_string(),
            }))
        }
    }

    PostIndexesBulkIndexResponse::Ok(Json(PostIndexBulkResponseOk {
        statuses: document_insert_statuses,
    }))
}

fn bulk_request_doc_to_tantivy_doc(
    bulk_request_doc: BulkRequestDoc,
    smiles_field: Field,
    fingerprint_field: Field,
    descriptors_fields: &HashMap<&str, Field>,
    extra_data_field: Field,
) -> Result<tantivy::Document, String> {
    // By default, do not attempt to fix problematic molecules
    let (tautomer, fingerprint, descriptors) =
        process_cpd(&bulk_request_doc.smiles, false).map_err(|err| err.to_string())?;

    let json: serde_json::Value = serde_json::to_value(descriptors).map_err(|x| x.to_string())?;
    let jsonified_compound_descriptors: Map<String, Value> =
        if let serde_json::Value::Object(map) = json {
            map
        } else {
            return Err("not an object".to_string());
        };

    let mut doc = tantivy::doc!(
        smiles_field => tautomer.as_smiles(),
        fingerprint_field => fingerprint.0.into_vec()
    );

    // TODO: remove clone() by passing request doc by value in to this function
    if let Some(extra_data) = bulk_request_doc.extra_data {
        doc.add_field_value(extra_data_field, extra_data);
    }

    for field in KNOWN_DESCRIPTORS {
        if let Some(serde_json::Value::Number(val)) = jsonified_compound_descriptors.get(field) {
            if field.starts_with("Num") || field.starts_with("lipinski") {
                let int = val.as_f64().unwrap() as i64;
                doc.add_field_value(*descriptors_fields.get(field).unwrap(), int);
            } else {
                doc.add_field_value(
                    *descriptors_fields.get(field).unwrap(),
                    val.as_f64().unwrap(),
                );
            };
        }
    }

    Ok(doc)
}
