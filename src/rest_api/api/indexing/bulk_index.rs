use crate::indexing::{combine_json_objects, index_manager::IndexManager, KNOWN_DESCRIPTORS};
use crate::rest_api::api::{
    BulkRequest, PostIndexBulkResponseError, PostIndexBulkResponseOk,
    PostIndexBulkResponseOkStatus, PostIndexesBulkIndexResponse,
};
use crate::search::compound_processing::process_cpd;
use crate::search::scaffold_search::{scaffold_search, PARSED_SCAFFOLDS};
use crate::search::similarity_search::encode_fingerprints;
use poem_openapi::payload::Json;
use rayon::prelude::*;
use serde_json::Value;
use std::collections::HashMap;
use bitvec::prelude::BitVec;
use rdkit::{Fingerprint, ROMol};
use tantivy::{doc, Document};
use tantivy::schema::{Field, Schema};

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

    let tantivy_docs_conversion_operation = tokio::task::spawn_blocking(move || {
        batch_doc_creation(bulk_request, &index.schema())
    })
    .await;

    let tantivy_docs = match tantivy_docs_conversion_operation {
        Ok(Ok(docs)) => docs,
        Ok(Err(e)) => return PostIndexesBulkIndexResponse::Err(Json(PostIndexBulkResponseError {
            error: e.to_string(),
        })),
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

fn batch_doc_creation(
    bulk_request: BulkRequest,
    schema: &Schema,
) -> eyre::Result<Vec<eyre::Result<impl Document>>> {
    let smiles_field = schema.get_field("smiles").unwrap();
    let pattern_fingerprint_field = schema.get_field("pattern_fingerprint").unwrap();
    let morgan_fingerprint_field = schema.get_field("morgan_fingerprint").unwrap();
    let extra_data_field = schema.get_field("extra_data").unwrap();
    let other_descriptors_field = schema.get_field("other_descriptors").unwrap();

    let descriptor_fields = KNOWN_DESCRIPTORS
        .iter()
        .map(|kd| (*kd, schema.get_field(kd).unwrap()))
        .collect::<HashMap<&str, Field>>();

    let mol_attributes = bulk_request
        .docs
        .into_par_iter()
        .map(|doc| {
            match process_cpd(&doc.smiles, false) {
                Ok(attributes) => {
                    (true, attributes.0, doc.extra_data, attributes.1, attributes.2)
                },
                Err(_) => {
                    let placeholder = process_cpd("c1ccccc1", false).unwrap();
                    (false, placeholder.0, None, placeholder.1, placeholder.2)
                }
            }
        })
        .collect::<Vec<_>>();

    let mut morgan_fingerprints: Vec<Fingerprint> = Vec::with_capacity(mol_attributes.len());
    let mut morgan_bitvecs: Vec<BitVec<u8>> = Vec::with_capacity(mol_attributes.len());
    for attributes in &mol_attributes {
        let morgan_fp = attributes.1.morgan_fingerprint();
        morgan_fingerprints.push(morgan_fp.clone());
        morgan_bitvecs.push(morgan_fp.0);
    }

    let similarity_clusters = encode_fingerprints(&morgan_bitvecs, true)
        .map_err(|e| eyre::eyre!("Failed batched similarity cluster assignment: {e}"))?;

    let docs = (0..mol_attributes.len())
        .into_iter()
        .map(|i| {
            let attributes = &mol_attributes[i];
            match attributes.0 {
                true => {
                    create_tantivy_doc(
                        &attributes.1,
                        &attributes.2,
                        &attributes.3,
                        &morgan_fingerprints[i],
                        &attributes.4,
                        similarity_clusters[i],
                        smiles_field,
                        pattern_fingerprint_field,
                        morgan_fingerprint_field,
                        &descriptor_fields,
                        extra_data_field,
                        other_descriptors_field,
                    )
                },
                false => Err(eyre::eyre!("Compound processing failed")),
            }
        }).collect::<Vec<_>>();

    Ok(docs)
}

fn create_tantivy_doc(
    canon_taut: &ROMol,
    extra_data: &Option<Value>,
    pattern_fp: &Fingerprint,
    morgan_fp: &Fingerprint,
    descriptors: &HashMap<String, f64>,
    similarity_cluster: i32,
    smiles_field: Field,
    pattern_fingerprint_field: Field,
    morgan_fingerprint_field: Field,
    descriptor_fields: &HashMap<&str, Field>,
    extra_data_field: Field,
    other_descriptors_field: Field,
) -> eyre::Result<impl Document> {
    let mut doc = doc!(
        smiles_field => canon_taut.as_smiles(),
        pattern_fingerprint_field => pattern_fp.0.as_raw_slice(),
        morgan_fingerprint_field => morgan_fp.0.as_raw_slice(),
    );

    let scaffold_matches = scaffold_search(&pattern_fp.0, &canon_taut, &PARSED_SCAFFOLDS)?;
    let scaffold_json = match scaffold_matches.is_empty() {
        true => serde_json::json!({"scaffolds": vec![-1]}),
        false => serde_json::json!({"scaffolds": scaffold_matches}),
    };

    let cluster_json = serde_json::json!({"similarity_cluster": similarity_cluster});

    let other_descriptors_json = combine_json_objects(Some(scaffold_json), Some(cluster_json));

    if let Some(other_descriptors_json) = other_descriptors_json {
        doc.add_field_value(other_descriptors_field, other_descriptors_json);
    }

    if let Some(extra_data) = extra_data {
        doc.add_field_value(extra_data_field, extra_data.clone());
    }

    for field in KNOWN_DESCRIPTORS {
        if let Some(val) = descriptors.get(field) {
            if field.starts_with("Num") || field.starts_with("lipinski") {
                let int = *val as i64;
                doc.add_field_value(*descriptor_fields.get(field).unwrap(), int);
            } else {
                doc.add_field_value(*descriptor_fields.get(field).unwrap(), *val);
            };
        }
    }

    Ok(doc)
}
