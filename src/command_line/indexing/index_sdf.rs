use bitvec::macros::internal::funty::Fundamental;
use rayon::prelude::*;
use rdkit::{Fingerprint, MolBlockIter, ROMol, RWMol};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use bitvec::prelude::BitVec;
use tantivy::Document;
use tantivy::schema::Field;

use crate::command_line::prelude::*;
use crate::search::compound_processing::process_cpd;
use crate::search::scaffold_search::{scaffold_search, PARSED_SCAFFOLDS};
use crate::search::similarity_search::encode_fingerprints;

pub const NAME: &str = "index-sdf";

pub fn command() -> Command {
    Command::new(NAME)
        .arg(
            Arg::new("sdf")
                .required(true)
                .long("sdf")
                .short('s')
                .num_args(1),
        )
        .arg(
            Arg::new("index")
                .required(true)
                .long("index")
                .short('i')
                .num_args(1),
        )
        .arg(
            Arg::new("limit")
                .required(false)
                .long("limit")
                .short('l')
                .num_args(1),
        )
        .arg(
            Arg::new("chunk-size")
                .required(false)
                .long("chunk-size")
                .num_args(1),
        )
        .arg(
            Arg::new("commit")
                .required(false)
                .long("commit")
                .num_args(0),
        )
}

pub fn action(matches: &ArgMatches) -> eyre::Result<()> {
    let sdf_path = matches
        .get_one::<String>("sdf")
        .ok_or(eyre::eyre!("Failed to extract sdf path"))?;
    let index_dir = matches
        .get_one::<String>("index")
        .ok_or(eyre::eyre!("Failed to extract index path"))?;
    let limit = matches.get_one::<String>("limit");
    let chunksize: usize = *matches.get_one("chunk-size").unwrap_or(&1000); // TODO figure out how to parse usize from CLI flags
    let commit: bool = matches.get_flag("commit");

    log::info!(
        "indexing path={}, index_dir={}, limit={:?}",
        sdf_path,
        index_dir,
        limit
    );

    let index_dir_metadata = std::fs::metadata(index_dir);
    if let Ok(metadata) = index_dir_metadata {
        if metadata.is_dir() {
            std::fs::remove_dir_all(index_dir)?;
        }
    }
    std::fs::create_dir(index_dir)?;

    let mol_iter = MolBlockIter::from_gz_file(sdf_path, true, true, false)
        .map_err(|e| eyre::eyre!("could not read gz file: {:?}", e))?;

    let mol_iter: Box<dyn Iterator<Item = Result<RWMol, String>>> = if let Some(limit) = limit {
        Box::new(mol_iter.take(limit.parse()?))
    } else {
        Box::new(mol_iter)
    };

    let schema = crate::schema::LIBRARY
        .get("descriptor_v1")
        .ok_or(eyre::eyre!("Failed to extract schema"))?;
    let index = create_or_reset_index(index_dir, schema)?;
    let mut index_writer = index.writer_with_num_threads(1, 50 * 1024 * 1024)?;

    // Get all relevant descriptor fields
    let smiles_field = schema.get_field("smiles")?;
    let pattern_fingerprint_field = schema.get_field("pattern_fingerprint")?;
    let morgan_fingerprint_field = schema.get_field("morgan_fingerprint")?;
    let other_descriptors_field = schema.get_field("other_descriptors")?;
    let descriptor_fields = KNOWN_DESCRIPTORS
        .iter()
        .map(|kd| (*kd, schema.get_field(kd).unwrap()))
        .collect::<HashMap<&str, Field>>();

    let mut counter = 0;
    let failed_counter: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));

    let mut mol_vec = Vec::with_capacity(chunksize);

    for mol in mol_iter {
        if mol.is_err() {
            let mut num = failed_counter.lock().unwrap();
            *num += 1;
            continue;
        }

        let mol = mol.unwrap();
        mol_vec.push(mol.to_ro_mol());

        if mol_vec.len() == chunksize {
            let doc_batch_result = batch_doc_creation(
                &mut mol_vec,
                &failed_counter,
                smiles_field,
                pattern_fingerprint_field,
                morgan_fingerprint_field,
                &descriptor_fields,
                other_descriptors_field,
            );

            match doc_batch_result {
                Err(e) => log::warn!("Failed batched doc creation: {e}"),
                Ok(doc_batch) => {
                    let _ = doc_batch
                        .into_par_iter()
                        .map(|doc| {
                            match index_writer.add_document(doc) {
                                Ok(_) => (),
                                Err(_) => {
                                    log::warn!("Failed doc creation");
                                    let mut num = failed_counter.lock().unwrap();
                                    *num += 1;
                                }
                            }
                        }).collect::<Vec<()>>();

                    if commit {
                        index_writer.commit()?;
                    }
                }
            }

            counter += chunksize;

            if counter > 0 && counter % 10_000 == 0 {
                log::info!("{:?} compounds processed so far", counter);
            }
        }
    }

    if !mol_vec.is_empty() {
        let last_chunksize = mol_vec.len();

        let doc_batch_result = batch_doc_creation(
            &mut mol_vec,
            &failed_counter,
            smiles_field,
            pattern_fingerprint_field,
            morgan_fingerprint_field,
            &descriptor_fields,
            other_descriptors_field,
        );

        match doc_batch_result {
            Err(e) => log::warn!("{e}"),
            Ok(doc_batch) => {
                let _ = doc_batch
                    .into_par_iter()
                    .map(|doc| {
                        match index_writer.add_document(doc) {
                            Ok(_) => (),
                            Err(_) => {
                                log::warn!("Failed doc creation");
                                let mut num = failed_counter.lock().unwrap();
                                *num += 1;
                            }
                        }
                    }).collect::<Vec<()>>();
            }
        }

        counter += last_chunksize;
    }

    let _ = index_writer.commit();

    log::info!(
        "A total of {:?} compounds were processed. Of those, {:?} compounds could not be indexed.",
        counter,
        failed_counter.lock().unwrap()
    );

    Ok(())
}

fn batch_doc_creation(
    mol_vec: &mut Vec<ROMol>,
    failed_counter: &Arc<Mutex<usize>>,
    smiles_field: Field,
    pattern_fingerprint_field: Field,
    morgan_fingerprint_field: Field,
    descriptor_fields: &HashMap<&str, Field>,
    other_descriptors_field: Field,
) -> eyre::Result<Vec<impl Document>> {
    let mol_attributes = mol_vec
        .clone()
        .into_par_iter()
        .filter_map(|m| {
            match process_cpd(m.as_smiles().as_str(), false) {
                Ok(attributes) => Some(attributes),
                Err(e) => {
                    log::warn!("Failed compound processing: {}", e);
                    let mut num = failed_counter.lock().unwrap();
                    *num += 1;
                    None
                }
            }
        }).collect::<Vec<(ROMol, Fingerprint, HashMap<String, f64>)>>();

    mol_vec.clear();

    let mut morgan_fingerprints: Vec<Fingerprint> = Vec::with_capacity(mol_attributes.len());
    let mut morgan_bitvecs: Vec<BitVec<u8>> = Vec::with_capacity(mol_attributes.len());
    for attributes in mol_attributes.clone() {
        let morgan_fp = attributes.0.morgan_fingerprint();
        morgan_fingerprints.push(morgan_fp.clone());
        morgan_bitvecs.push(morgan_fp.0);
    }

    let similarity_clusters = encode_fingerprints(&morgan_bitvecs, true);

    if let Err(e) = similarity_clusters {
        let mut num = failed_counter.lock().unwrap();
        *num += morgan_bitvecs.len();
        return Err(eyre::eyre!("Failed batched similarity cluster assignment: {e}"))
    }

    let similarity_clusters = similarity_clusters.unwrap();

    let docs = (0..mol_attributes.len())
        .into_iter()
        .filter_map(|i| {
            match create_tantivy_doc(
                mol_attributes[i].0.to_owned(),
                mol_attributes[i].1.to_owned(),
                morgan_fingerprints[i].to_owned(),
                mol_attributes[i].2.to_owned(),
                similarity_clusters[i],
                smiles_field,
                pattern_fingerprint_field,
                morgan_fingerprint_field,
                &descriptor_fields,
                other_descriptors_field,
            ) {
                Ok(doc) => Some(doc),
                Err(_) => {
                    log::warn!("Failed doc creation");
                    let mut num = failed_counter.lock().unwrap();
                    *num += 1;
                    None
                },
            }
        }).collect::<Vec<_>>();

    Ok(docs)
}

fn create_tantivy_doc(
    canon_taut: ROMol,
    pattern_fp: Fingerprint,
    morgan_fp: Fingerprint,
    descriptors: HashMap<String, f64>,
    similarity_cluster: i32,
    smiles_field: Field,
    pattern_fingerprint_field: Field,
    morgan_fingerprint_field: Field,
    descriptor_fields: &HashMap<&str, Field>,
    other_descriptors_field: Field,
) -> eyre::Result<impl Document> {
    let mut doc = doc!(
        smiles_field => canon_taut.as_smiles(),
        pattern_fingerprint_field => pattern_fp.0.as_raw_slice(),
        morgan_fingerprint_field => morgan_fp.0.as_raw_slice()
    );

    for field in KNOWN_DESCRIPTORS {
        if let Some(val) = descriptors.get(field) {
            let current_field = *descriptor_fields
                .get(field)
                .ok_or(eyre::eyre!("Failed to extract field"))?;
            if field.starts_with("Num") || field.starts_with("lipinski") {
                let int = val.as_f64() as i64;
                doc.add_field_value(current_field, int);
            } else {
                doc.add_field_value(current_field, val.as_f64());
            };
        }
    }

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

    Ok(doc)
}
