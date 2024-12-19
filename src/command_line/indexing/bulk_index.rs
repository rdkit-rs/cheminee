use crate::command_line::{indexing::split_path, prelude::*};
use crate::indexing::index_manager::IndexManager;
use crate::search::compound_processing::process_cpd;
use crate::search::similarity_search::encode_fingerprints;
use rayon::prelude::*;
use std::{collections::HashMap, fs::File, io::BufRead, io::BufReader, ops::Deref};
use bitvec::prelude::BitVec;
use rdkit::{Fingerprint, ROMol};
use serde_json::Value;
use tantivy::Document;
use tantivy::schema::{Field, Schema};

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

    let file = File::open(json_path)?;
    let reader = BufReader::new(file);
    let chunksize = 1000;
    let mut record_vec = Vec::with_capacity(chunksize);

    for result_line in reader.lines() {
        let line = result_line?;
        let record = serde_json::from_str(&line)?;

        record_vec.push(record);
        if record_vec.len() == chunksize {
            match batch_doc_creation(&mut record_vec, &schema) {
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
        match batch_doc_creation(&mut record_vec, &schema) {
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
    schema: &Schema,
) -> eyre::Result<Vec<impl Document>> {
    let smiles_field = schema.get_field("smiles")?;
    let pattern_fingerprint_field = schema.get_field("pattern_fingerprint")?;
    let morgan_fingerprint_field = schema.get_field("morgan_fingerprint")?;
    let extra_data_field = schema.get_field("extra_data")?;
    let other_descriptors_field = schema.get_field("other_descriptors")?;
    let descriptor_fields = KNOWN_DESCRIPTORS
        .iter()
        .map(|kd| (*kd, schema.get_field(kd).unwrap()))
        .collect::<HashMap<&str, Field>>();

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
            let attributes = &mol_attributes[i];
            match create_tantivy_doc(
                &attributes.0,
                &attributes.1,
                &attributes.2,
                &morgan_fingerprints[i],
                &attributes.3,
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
