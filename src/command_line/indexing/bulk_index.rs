use crate::command_line::{indexing::split_path, prelude::*};
use crate::indexing::index_manager::IndexManager;
use rayon::prelude::*;
use serde_json::Value;
use std::{fs::File, io::BufRead, io::BufReader, ops::Deref};

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
    let mut writer = index.writer(50 * 1024 * 1024)?;
    let schema = index.schema();

    let file = File::open(json_path)?;
    let reader = BufReader::new(file);
    let chunksize = 1000;
    let mut compound_vec = Vec::with_capacity(chunksize);

    for result_line in reader.lines() {
        let line = result_line?;
        let record = &serde_json::from_str(&line)?;
        let smiles_and_extra_data = get_smiles_and_extra_data(record)?;

        compound_vec.push(smiles_and_extra_data);
        if compound_vec.len() == chunksize {
            match batch_doc_creation(&compound_vec, &schema) {
                Err(e) => log::warn!("Failed batched doc creation: {e}"),
                Ok(doc_batch) => {
                    let _ = doc_batch
                        .into_par_iter()
                        .map(|doc| match doc {
                            Ok(doc) => match writer.add_document(doc) {
                                Ok(_) => (),
                                Err(_) => {
                                    log::warn!("Failed doc creation: Could not add document");
                                }
                            },
                            Err(e) => {
                                log::warn!("Failed doc creation: {e}");
                            }
                        })
                        .collect::<Vec<()>>();
                }
            }

            compound_vec.clear();
        }
    }

    if !compound_vec.is_empty() {
        match batch_doc_creation(&compound_vec, &schema) {
            Err(e) => log::warn!("Doc creation batch failed: {e}"),
            Ok(doc_batch) => {
                let _ = doc_batch
                    .into_par_iter()
                    .map(|doc| match doc {
                        Ok(doc) => match writer.add_document(doc) {
                            Ok(_) => (),
                            Err(_) => {
                                log::warn!("Failed doc creation: Could not add document");
                            }
                        },
                        Err(e) => {
                            log::warn!("Failed doc creation: {e}");
                        }
                    })
                    .collect::<Vec<()>>();
            }
        }

        compound_vec.clear();
    }

    let _ = writer.commit();

    Ok(())
}

fn get_smiles_and_extra_data(record: &Value) -> eyre::Result<(String, Option<Value>)> {
    let smiles = record
        .get("smiles")
        .ok_or(eyre::eyre!("Failed to extract smiles"))?
        .as_str()
        .ok_or(eyre::eyre!("Failed to parse smiles"))?
        .to_string();
    let extra_data = record.get("extra_data").cloned();
    Ok((smiles, extra_data))
}
