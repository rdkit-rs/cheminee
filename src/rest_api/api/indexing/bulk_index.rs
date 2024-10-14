use crate::indexing::{combine_json_objects, index_manager::IndexManager, KNOWN_DESCRIPTORS};
use crate::rest_api::api::{
    BulkRequest, BulkRequestDoc, PostIndexBulkResponseError, PostIndexBulkResponseOk,
    PostIndexBulkResponseOkStatus, PostIndexesBulkIndexResponse,
};
use crate::search::compound_processing::process_cpd;
use crate::search::scaffold_search::{scaffold_search, PARSED_SCAFFOLDS};
use poem_openapi::payload::Json;
use rayon::prelude::*;
use serde_json::{Map, Value};
use std::collections::HashMap;
use tantivy::doc;
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

    let mut writer = match index.writer(16 * 1024 * 1024) {
        Ok(writer) => writer,
        Err(e) => {
            return PostIndexesBulkIndexResponse::Err(Json(PostIndexBulkResponseError {
                error: e.to_string(),
            }))
        }
    };

    let schema = index.schema();

    let smiles_field = schema.get_field("smiles").unwrap();
    let pattern_fingerprint_field = schema.get_field("pattern_fingerprint").unwrap();
    let morgan_fingerprint_field = schema.get_field("morgan_fingerprint").unwrap();
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
                    pattern_fingerprint_field,
                    morgan_fingerprint_field,
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
    pattern_fingerprint_field: Field,
    morgan_fingerprint_field: Field,
    descriptors_fields: &HashMap<&str, Field>,
    extra_data_field: Field,
) -> eyre::Result<impl tantivy::Document> {
    // By default, do not attempt to fix problematic molecules
    let (canon_taut, pattern_fingerprint, descriptors) =
        process_cpd(&bulk_request_doc.smiles, false)?;

    let json: Value = serde_json::to_value(descriptors)?;
    let jsonified_compound_descriptors: Map<String, Value> = if let Value::Object(map) = json {
        map
    } else {
        return Err(eyre::eyre!("not an object"));
    };

    let mut doc = tantivy::doc!(
        smiles_field => canon_taut.as_smiles(),
        pattern_fingerprint_field => pattern_fingerprint.0.as_raw_slice(),
        morgan_fingerprint_field => canon_taut.morgan_fingerprint().0.as_raw_slice(),
    );

    let scaffolds = &PARSED_SCAFFOLDS;
    let scaffold_matches = scaffold_search(&pattern_fingerprint.0, &canon_taut, scaffolds)?;

    let scaffold_json = match scaffold_matches.is_empty() {
        true => serde_json::json!({"scaffolds": vec![-1]}),
        false => serde_json::json!({"scaffolds": scaffold_matches}),
    };

    let extra_data_json = combine_json_objects(Some(scaffold_json), bulk_request_doc.extra_data);
    if let Some(extra_data_json) = extra_data_json {
        doc.add_field_value(extra_data_field, extra_data_json);
    }

    for field in KNOWN_DESCRIPTORS {
        if let Some(Value::Number(val)) = jsonified_compound_descriptors.get(field) {
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
