use bitvec::macros::internal::funty::Fundamental;
use rayon::prelude::*;
use rdkit::{MolBlockIter, ROMol, RWMol};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tantivy::schema::Field;

use crate::command_line::prelude::*;
use crate::search::compound_processing::process_cpd;
use crate::search::scaffold_search::{scaffold_search, PARSED_SCAFFOLDS};
use crate::search::similarity_search::encode_fingerprint;

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
            let _ = mol_vec
                .clone()
                .into_par_iter()
                .map(|m| {
                    let doc = create_tantivy_doc(
                        m,
                        smiles_field,
                        pattern_fingerprint_field,
                        morgan_fingerprint_field,
                        &descriptor_fields,
                        other_descriptors_field,
                    );

                    match doc {
                        Ok(doc) => {
                            let write_operation = index_writer.add_document(doc);

                            match write_operation {
                                Ok(_) => (),
                                Err(_) => {
                                    log::warn!("Failed doc creation");
                                    let mut num = failed_counter.lock().unwrap();
                                    *num += 1;
                                }
                            }
                        }
                        Err(_) => {
                            log::warn!("Failed doc creation");
                            let mut num = failed_counter.lock().unwrap();
                            *num += 1;
                        }
                    }
                })
                .collect::<Vec<_>>();

            mol_vec.clear();
            counter += chunksize;

            if commit {
                index_writer.commit()?;
            }

            if counter > 0 && counter % 10_000 == 0 {
                log::info!("{:?} compounds processed so far", counter);
            }
        }
    }

    if !mol_vec.is_empty() {
        let _ = mol_vec
            .clone()
            .into_par_iter()
            .map(|m| {
                let doc = create_tantivy_doc(
                    m,
                    smiles_field,
                    pattern_fingerprint_field,
                    morgan_fingerprint_field,
                    &descriptor_fields,
                    other_descriptors_field,
                );

                match doc {
                    Ok(doc) => {
                        let write_operation = index_writer.add_document(doc);

                        match write_operation {
                            Ok(_) => (),
                            Err(_) => {
                                log::warn!("Failed doc creation");
                                let mut num = failed_counter.lock().unwrap();
                                *num += 1;
                            }
                        }
                    }
                    Err(_) => {
                        log::warn!("Failed doc creation");
                        let mut num = failed_counter.lock().unwrap();
                        *num += 1;
                    }
                }
            })
            .collect::<Vec<_>>();

        counter += mol_vec.len();
    }

    let _ = index_writer.commit();

    log::info!(
        "A total of {:?} compounds were processed. Of those, {:?} compounds could not be indexed.",
        counter,
        failed_counter.lock().unwrap()
    );

    Ok(())
}

fn create_tantivy_doc(
    mol: ROMol,
    smiles_field: Field,
    pattern_fingerprint_field: Field,
    morgan_fingerprint_field: Field,
    descriptor_fields: &HashMap<&str, Field>,
    other_descriptors_field: Field,
) -> eyre::Result<impl tantivy::Document> {
    // By default, do not attempt to fix problematic molecules
    let (canon_taut, pattern_fp, descriptors) = process_cpd(mol.as_smiles().as_str(), false)?;

    let morgan_fp = canon_taut.morgan_fingerprint();

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

    let similarity_cluster = encode_fingerprint(&morgan_fp.0, true)?[0];
    let cluster_json = serde_json::json!({"similarity_cluster": similarity_cluster});

    let other_descriptors_json = combine_json_objects(Some(scaffold_json), Some(cluster_json));

    if let Some(other_descriptors_json) = other_descriptors_json {
        doc.add_field_value(other_descriptors_field, other_descriptors_json);
    }

    Ok(doc)
}
