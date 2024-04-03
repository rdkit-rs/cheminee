use crate::command_line::indexing::split_path;
use crate::command_line::prelude::*;
use crate::indexing::index_manager::IndexManager;
use crate::search::compound_processing::standardize_smiles;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Deref;

pub const NAME: &str = "index-scaffolds";
pub fn command() -> Command {
    Command::new(NAME)
        .arg(
            Arg::new("json-path")
                .required(true)
                .long("json-path")
                .short('j')
                .num_args(1),
        )
        .arg(
            Arg::new("index-path")
                .required(true)
                .long("index-path")
                .short('i')
                .num_args(1),
        )
}

pub fn action(matches: &ArgMatches) -> eyre::Result<u64> {
    let json_path = matches
        .get_one::<String>("json-path")
        .ok_or(eyre::eyre!("Failed to extract json path"))?;
    let index_path = matches
        .get_one::<String>("index-path")
        .ok_or(eyre::eyre!("Failed to extract index path"))?;
    let (storage_dir, index_name) = split_path(index_path)?;
    let index_manager = IndexManager::new(storage_dir.deref(), false)?;

    let schema = crate::schema::LIBRARY
        .get("scaffold")
        .ok_or(eyre::eyre!("The specified schema does not exist"))?;

    let index = index_manager.create(index_name.deref(), schema, true, None)?;
    let mut writer = index.writer(15_000_000)?;

    let smiles_field = schema.get_field("smiles")?;
    let id_field = schema.get_field("id")?;

    let file = File::open(json_path)?;
    let reader = BufReader::new(file);
    let mut scaffold_id: u64 = 0;
    for result_line in reader.lines() {
        let line = result_line?;
        let record: serde_json::Value = serde_json::from_str(&line)?;
        let raw_smiles = record
            .get("smiles")
            .ok_or(eyre::eyre!("Failed to extract smiles"))?
            .as_str()
            .ok_or(eyre::eyre!("Failed to convert smiles to str"))?;

        let romol = standardize_smiles(raw_smiles, true);
        match romol {
            Ok(romol) => {
                let doc = doc!(
                    smiles_field => romol.as_smiles(),
                    id_field => scaffold_id
                );

                let write_operation = writer.add_document(doc);
                match write_operation {
                    Ok(_) => scaffold_id += 1,
                    Err(e) => println!("Failed doc creation: {:?}", e),
                }
            }
            Err(e) => println!("Failed compound standardization: {:?}", e),
        }
    }

    let _ = writer.commit();
    Ok(scaffold_id + 1)
}
