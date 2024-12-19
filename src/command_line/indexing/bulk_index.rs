use crate::command_line::{indexing::split_path, prelude::*};
use crate::indexing::index_manager::IndexManager;
use crate::search::compound_processing::process_cpd;
use crate::search::scaffold_search::{scaffold_search, PARSED_SCAFFOLDS};
use crate::search::similarity_search::encode_fingerprints;
use rayon::prelude::*;
use std::{collections::HashMap, fs::File, io::BufRead, io::BufReader, ops::Deref};
use bitvec::prelude::BitVec;
use rdkit::{Fingerprint, ROMol};
use serde_json::Value;
use tantivy::Document;
use tantivy::schema::Field;

pub const NAME: &str = "bulk-index";

pub fn command() -> Command {
    Command::new(NAME)
        .arg(
            Arg::new("index-path")
                .required(true)
                .long("index-path")
                .short('i')
                .num_args(1),
        )
        .arg(
            Arg::new("json-path")
                .required(true)
                .long("json-path")
                .short('j')
                .num_args(1),
        )
}

pub fn action(matches: &ArgMatches) -> eyre::Result<()> {
    let index_path = matches
        .get_one::<String>("index-path")
        .ok_or(eyre::eyre!("Failed to extract index path"))?;
    let json_path = matches
        .get_one::<String>("json-path")
        .ok_or(eyre::eyre!("Failed to extract json path"))?;

    let (storage_dir, index_name) = split_path(index_path)?;
    let index_manager = IndexManager::new(storage_dir.deref(), false)?;

    let index = index_manager.open(index_name.deref())?;
    let mut writer = index.writer(16 * 1024 * 1024)?;
    let schema = index.schema();

    let smiles_field = schema.get_field("smiles")?;
    let pattern_fingerprint_field = schema.get_field("pattern_fingerprint")?;
    let morgan_fingerprint_field = schema.get_field("morgan_fingerprint")?;
    let extra_data_field = schema.get_field("extra_data")?;
    let other_descriptors_field = schema.get_field("other_descriptors")?;
    let descriptor_fields = KNOWN_DESCRIPTORS
        .iter()
        .map(|kd| (*kd, schema.get_field(kd).unwrap()))
        .collect::<HashMap<&str, Field>>();

    let file = File::open(json_path)?;
    let reader = BufReader::new(file);
    let chunksize = 1000;
    let mut record_vec = Vec::with_capacity(chunksize);

    for result_line in reader.lines() {
        let line = result_line?;
        let record = serde_json::from_str(&line)?;

        record_vec.push(record);
        if record_vec.len() == chunksize {
            let doc_batch_result = batch_doc_creation(
                &mut record_vec,
                smiles_field,
                extra_data_field,
                pattern_fingerprint_field,
                morgan_fingerprint_field,
                &descriptor_fields,
                other_descriptors_field,
            );

            match doc_batch_result {
                Err(e) => log::warn!("Doc creation batch failed: {e}"),
                Ok(doc_batch) => {
                    let _ = doc_batch
                        .into_par_iter()
                        .map(|doc| {
                            match writer.add_document(doc) {
                                Ok(_) => (),
                                Err(_) => {
                                    log::warn!("Failed doc creation");
                                }
                            }
                        }).collect::<Vec<()>>();
                }
            }
        }
    }

    if !record_vec.is_empty() {
        let doc_batch_result = batch_doc_creation(
            &mut record_vec,
            smiles_field,
            extra_data_field,
            pattern_fingerprint_field,
            morgan_fingerprint_field,
            &descriptor_fields,
            other_descriptors_field,
        );

        match doc_batch_result {
            Err(e) => log::warn!("Doc creation batch failed: {e}"),
            Ok(doc_batch) => {
                let _ = doc_batch
                    .into_par_iter()
                    .map(|doc| {
                        match writer.add_document(doc) {
                            Ok(_) => (),
                            Err(_) => {
                                log::warn!("Failed doc creation");
                            }
                        }
                    }).collect::<Vec<()>>();
            }
        }
    }

    let _ = writer.commit();

    Ok(())
}

fn batch_doc_creation(
    record_vec: &mut Vec<Value>,
    smiles_field: Field,
    extra_data_field: Field,
    pattern_fingerprint_field: Field,
    morgan_fingerprint_field: Field,
    descriptor_fields: &HashMap<&str, Field>,
    other_descriptors_field: Field,
) -> eyre::Result<Vec<impl Document>> {
    let mol_attributes = record_vec
        .clone()
        .into_par_iter()
        .filter_map(|r| {
            let extra_data = r.get("extra_data").cloned();
            if let Some(smiles) = r.get("smiles") {
                if let Some(smiles) = smiles.as_str() {
                    match process_cpd(smiles, false) {
                        Ok(attributes) => {
                            Some((attributes.0, extra_data, attributes.1, attributes.2))
                        },
                        Err(e) => {
                            log::warn!("Failed compound processing for smiles '{}': {}", smiles, e);
                            None
                        }
                    }
                } else {
                    log::warn!("Failed to convert smiles to str");
                    None
                }
            } else {
                log::warn!("Failed to extract smiles");
                None
            }
        }).collect::<Vec<(ROMol, Option<Value>, Fingerprint, HashMap<String, f64>)>>();

    record_vec.clear();

    let mut morgan_fingerprints: Vec<Fingerprint> = Vec::with_capacity(mol_attributes.len());
    let mut morgan_bitvecs: Vec<BitVec<u8>> = Vec::with_capacity(mol_attributes.len());
    for attributes in mol_attributes.clone() {
        let morgan_fp = attributes.0.morgan_fingerprint();
        morgan_fingerprints.push(morgan_fp.clone());
        morgan_bitvecs.push(morgan_fp.0);
    }

    let similarity_clusters = encode_fingerprints(&morgan_bitvecs, true)
        .map_err(|e| eyre::eyre!("Failed batched similarity cluster assignment: {e}"))?;

    let docs = (0..mol_attributes.len())
        .into_iter()
        .filter_map(|i| {
            match create_tantivy_doc(
                mol_attributes[i].0.to_owned(),
                mol_attributes[i].1.to_owned(),
                mol_attributes[i].2.to_owned(),
                morgan_fingerprints[i].to_owned(),
                mol_attributes[i].3.to_owned(),
                similarity_clusters[i],
                smiles_field,
                pattern_fingerprint_field,
                morgan_fingerprint_field,
                &descriptor_fields,
                extra_data_field,
                other_descriptors_field,
            ) {
                Ok(doc) => Some(doc),
                Err(_) => {
                    log::warn!("Failed doc creation");
                    None
                },
            }
        }).collect::<Vec<_>>();

    Ok(docs)
}

fn create_tantivy_doc(
    canon_taut: ROMol,
    extra_data: Option<Value>,
    pattern_fp: Fingerprint,
    morgan_fp: Fingerprint,
    descriptors: HashMap<String, f64>,
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
        doc.add_field_value(extra_data_field, extra_data);
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
