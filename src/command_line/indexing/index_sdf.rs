use bitvec::macros::internal::funty::Fundamental;
use rayon::prelude::*;
use rdkit::{MolBlockIter, ROMol, RWMol};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tantivy::schema::Field;

use crate::command_line::prelude::*;
use crate::search::compound_processing::process_cpd;
use crate::search::scaffold_search::{scaffold_search, PARSED_SCAFFOLDS};
use crate::search::similarity_search::assign_pca_bins;

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
}

pub fn action(matches: &ArgMatches) -> eyre::Result<()> {
    let sdf_path = matches
        .get_one::<String>("sdf")
        .ok_or(eyre::eyre!("Failed to extract sdf path"))?;
    let index_dir = matches
        .get_one::<String>("index")
        .ok_or(eyre::eyre!("Failed to extract index path"))?;
    let limit = matches.get_one::<String>("limit");

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

    let mol_iter = MolBlockIter::from_gz_file(sdf_path, false, false, false)
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
    let fingerprint_field = schema.get_field("fingerprint")?;
    let extra_data_field = schema.get_field("extra_data")?;
    let descriptor_fields = KNOWN_DESCRIPTORS
        .iter()
        .map(|kd| (*kd, schema.get_field(kd).unwrap()))
        .collect::<HashMap<&str, Field>>();

    let mut counter = 0;
    // let mut failed_counter = 0;
    let failed_counter: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));

    let chunksize = 1000;
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
                        fingerprint_field,
                        &descriptor_fields,
                        extra_data_field,
                    );

                    match doc {
                        Ok(doc) => {
                            let write_operation = index_writer.add_document(doc);

                            match write_operation {
                                Ok(_) => (),
                                Err(_) => {
                                    println!("Failed doc creation");
                                    let mut num = failed_counter.lock().unwrap();
                                    *num += 1;
                                }
                            }
                        }
                        Err(_) => {
                            println!("Failed doc creation");
                            let mut num = failed_counter.lock().unwrap();
                            *num += 1;
                        }
                    }
                })
                .collect::<Vec<_>>();

            index_writer.commit()?;
            mol_vec.clear();
            counter += chunksize;

            if counter > 0 && counter % 10_000 == 0 {
                println!("{:?} compounds processed so far", counter);
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
                    fingerprint_field,
                    &descriptor_fields,
                    extra_data_field,
                );

                match doc {
                    Ok(doc) => {
                        let write_operation = index_writer.add_document(doc);

                        match write_operation {
                            Ok(_) => (),
                            Err(_) => {
                                println!("Failed doc creation");
                                let mut num = failed_counter.lock().unwrap();
                                *num += 1;
                            }
                        }
                    }
                    Err(_) => {
                        println!("Failed doc creation");
                        let mut num = failed_counter.lock().unwrap();
                        *num += 1;
                    }
                }
            })
            .collect::<Vec<_>>();

        index_writer.commit()?;
        counter += mol_vec.len();
    }

    println!(
        "A total of {:?} compounds were processed. Of those, {:?} compounds could not be indexed.",
        counter,
        failed_counter.lock().unwrap()
    );

    Ok(())
}

fn create_tantivy_doc(
    mol: ROMol,
    smiles_field: Field,
    fingerprint_field: Field,
    descriptor_fields: &HashMap<&str, Field>,
    extra_data_field: Field,
) -> eyre::Result<tantivy::Document> {
    // By default, do not attempt to fix problematic molecules
    let (canon_taut, fp, descriptors) = process_cpd(mol.as_smiles().as_str(), false)?;

    let mut doc = doc!(
        smiles_field => canon_taut.as_smiles(),
        fingerprint_field => fp.0.into_vec()
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

    let scaffold_matches = scaffold_search(&canon_taut, &PARSED_SCAFFOLDS)?;

    let scaffold_json = match scaffold_matches.is_empty() {
        true => serde_json::json!({"scaffolds": vec![-1]}),
        false => serde_json::json!({"scaffolds": scaffold_matches}),
    };

    let pca_bins = assign_pca_bins(&descriptors)
        .iter()
        .enumerate()
        .map(|(idx, bin)| (format!("pc{idx}"), serde_json::json!(bin)))
        .collect();

    let pca_bins_json = Value::Object(pca_bins);

    let extra_data_json = combine_json_objects(Some(scaffold_json), Some(pca_bins_json));

    if let Some(extra_data_json) = extra_data_json {
        doc.add_field_value(extra_data_field, extra_data_json);
    }

    Ok(doc)
}
