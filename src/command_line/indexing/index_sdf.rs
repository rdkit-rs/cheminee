use rayon::prelude::*;
use rdkit::{Fingerprint, MolBlockIter, ROMol, RWMol};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use bitvec::prelude::BitVec;
use tantivy::Document;
use tantivy::schema::{Field, Schema};

use crate::command_line::prelude::*;
use crate::search::compound_processing::process_cpd;
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
            match batch_doc_creation(&mut mol_vec, &failed_counter, &schema) {
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

        match batch_doc_creation(&mut mol_vec, &failed_counter, &schema) {
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
            let attributes = &mol_attributes[i];
            match create_tantivy_doc(
                &attributes.0,
                &None,
                &attributes.1,
                &morgan_fingerprints[i],
                &attributes.2,
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
                    let mut num = failed_counter.lock().unwrap();
                    *num += 1;
                    None
                },
            }
        }).collect::<Vec<_>>();

    Ok(docs)
}
