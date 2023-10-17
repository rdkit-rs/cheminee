use std::collections::HashMap;

use poem_openapi::payload::Json;
use poem_openapi_derive::{ApiResponse, Object};
use serde_json::{Map, Value};
use tantivy::{schema::Field, Opstamp};

use crate::{
    indexing::{index_manager::IndexManager, KNOWN_DESCRIPTORS},
    search::compound_processing::process_cpd,
};

#[derive(ApiResponse)]
pub enum PostIndexesBulkIndexResponse {
    #[oai(status = "200")]
    Ok(Json<PostIndexBulkResponseOk>),
    #[oai(status = "404")]
    IndexDoesNotExist,
    #[oai(status = "500")]
    Err(Json<PostIndexBulkResponseError>),
}

#[derive(Object, Debug)]
pub struct BulkRequest {
    pub docs: Vec<BulkRequestDoc>,
}

#[derive(Object, Debug)]
pub struct BulkRequestDoc {
    pub smile: String,
    pub extra_data: Option<serde_json::Value>,
}

#[derive(Object, Debug)]
pub struct PostIndexBulkResponseOk {
    pub statuses: Vec<PostIndexBulkResponseOkStatus>,
}

#[derive(Object, Debug)]
pub struct PostIndexBulkResponseOkStatus {
    opcode: Option<Opstamp>,
    error: Option<String>,
}

#[derive(Object, Debug)]
pub struct PostIndexBulkResponseError {
    pub error: String,
}

pub fn v1_post_index_bulk(
    index_manager: &IndexManager,
    index: String,
    bulk_request: BulkRequest,
) -> PostIndexesBulkIndexResponse {
    match index_manager.exists(&index) {
        Ok(None) => return PostIndexesBulkIndexResponse::IndexDoesNotExist,
        Err(e) => {
            return PostIndexesBulkIndexResponse::Err(Json(PostIndexBulkResponseError {
                error: e.to_string(),
            }))
        }
        _ => (),
    }

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

    let mut document_insert_statuses = Vec::with_capacity(bulk_request.docs.len());

    let schema = index.schema();

    let smile_field = schema.get_field("smile").unwrap();
    let fingerprint_field = schema.get_field("fingerprint").unwrap();
    let extra_data_field = schema.get_field("extra_data").unwrap();

    let descriptors_fields = KNOWN_DESCRIPTORS
        .iter()
        .map(|kd| (*kd, schema.get_field(kd).unwrap()))
        .collect::<HashMap<&str, Field>>();

    for doc in bulk_request.docs {
        let tantivy_doc = bulk_request_doc_to_tantivy_doc(
            &doc,
            smile_field,
            fingerprint_field,
            &descriptors_fields,
            extra_data_field,
        );

        let tantivy_doc = match tantivy_doc {
            Ok(td) => td,
            Err(e) => {
                document_insert_statuses.push(PostIndexBulkResponseOkStatus {
                    opcode: None,
                    error: Some(e),
                });
                continue;
            }
        };

        let opstamp = writer.add_document(tantivy_doc);

        let status = match opstamp {
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
    bulk_request_doc: &BulkRequestDoc,
    smile_field: Field,
    fingerprint_field: Field,
    descriptors_fields: &HashMap<&str, Field>,
    extra_data_field: Field,
) -> Result<tantivy::Document, String> {
    let (tautomer, fingerprint, descriptors) =
        { process_cpd(&bulk_request_doc.smile).map_err(|err| err.to_string())? };

    let json: serde_json::Value = serde_json::to_value(&descriptors).map_err(|x| x.to_string())?;
    let jsonified_compound_descriptors: Map<String, Value> =
        if let serde_json::Value::Object(map) = json {
            map
        } else {
            return Err("not an object".to_string());
        };

    let mut doc = tantivy::doc!(
        smile_field => tautomer.as_smile(),
        fingerprint_field => fingerprint.0.into_vec()
    );

    // TODO: remove clone() by passing request doc by value in to this function
    if let Some(extra_data) = bulk_request_doc.extra_data.clone() {
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
