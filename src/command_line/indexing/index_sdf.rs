use crate::command_line::prelude::*;
use rayon::prelude::*;
use rdkit::{MolBlockIter, RWMol};
use std::sync::{Arc, Mutex};
use tantivy::directory::MmapDirectory;

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
                .short('c')
                .num_args(1),
        )
        .arg(
            Arg::new("create-or-reset-index")
                .required(false)
                .long("create-or-reset-index")
                .num_args(0),
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
    let chunksize = matches.get_one::<String>("chunk-size");
    let reset_index: bool = matches.get_flag("create-or-reset-index");
    let commit: bool = matches.get_flag("commit");

    let chunksize = if let Some(chunksize) = chunksize {
        chunksize.parse::<usize>()?
    } else {
        usize::try_from(1000)?
    };

    log::info!(
        "indexing path={}, index_dir={}, limit={:?}",
        sdf_path,
        index_dir,
        limit
    );

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

    let index = if reset_index {
        create_or_reset_index(index_dir, schema)?
    } else {
        let mmap_directory = MmapDirectory::open(index_dir)?;
        tantivy::Index::open(mmap_directory)?
    };

    let mut index_writer = index.writer(50 * 1024 * 1024)?;

    let mut counter = 0;
    let failed_counter: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));

    let mut compound_vec = Vec::with_capacity(chunksize);

    for mol in mol_iter {
        if mol.is_err() {
            let mut num = failed_counter.lock().unwrap();
            *num += 1;
            continue;
        }

        let mol = mol.unwrap();
        compound_vec.push((mol.to_ro_mol().as_smiles(), None));

        if compound_vec.len() == chunksize {
            match batch_doc_creation(&compound_vec, schema) {
                Err(e) => log::warn!("Failed batched doc creation: {e}"),
                Ok(doc_batch) => {
                    let _ = doc_batch
                        .into_par_iter()
                        .map(|doc| match doc {
                            Ok(doc) => match index_writer.add_document(doc) {
                                Ok(_) => (),
                                Err(_) => {
                                    log::warn!("Failed doc creation: Could not add document");
                                    let mut num = failed_counter.lock().unwrap();
                                    *num += 1;
                                }
                            },
                            Err(e) => {
                                log::warn!("Failed doc creation: {e}");
                                let mut num = failed_counter.lock().unwrap();
                                *num += 1;
                            }
                        })
                        .collect::<Vec<()>>();

                    if commit {
                        index_writer.commit()?;
                    }
                }
            }

            compound_vec.clear();
            counter += chunksize;

            if counter > 0 && counter % 10_000 == 0 {
                log::info!("{:?} compounds processed so far", counter);
            }
        }
    }

    if !compound_vec.is_empty() {
        let last_chunksize = compound_vec.len();
        match batch_doc_creation(&compound_vec, schema) {
            Err(e) => log::warn!("Failed batched doc creation: {e}"),
            Ok(doc_batch) => {
                let _ = doc_batch
                    .into_par_iter()
                    .map(|doc| match doc {
                        Ok(doc) => match index_writer.add_document(doc) {
                            Ok(_) => (),
                            Err(_) => {
                                log::warn!("Failed doc creation: Could not add document");
                                let mut num = failed_counter.lock().unwrap();
                                *num += 1;
                            }
                        },
                        Err(e) => {
                            log::warn!("Failed doc creation: {e}");
                            let mut num = failed_counter.lock().unwrap();
                            *num += 1;
                        }
                    })
                    .collect::<Vec<()>>();
            }
        }

        compound_vec.clear();
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
