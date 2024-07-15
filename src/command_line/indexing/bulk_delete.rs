use crate::command_line::{indexing::split_path, prelude::*};
use crate::indexing::index_manager::IndexManager;
use crate::search::compound_processing::process_cpd;
use crate::search::scaffold_search::{scaffold_search, PARSED_SCAFFOLDS};
use std::ops::Deref;
use tantivy::query::{Query, QueryParser};

pub const NAME: &str = "bulk-delete";

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
            Arg::new("smiles-list")
                .required(false)
                .long("smiles-list")
                .short('s')
                .num_args(1),
        )
}

pub fn action(matches: &ArgMatches) -> eyre::Result<()> {
    let index_path = matches
        .get_one::<String>("index-path")
        .ok_or(eyre::eyre!("Failed to extract index path"))?;
    let smiles_list = matches
        .get_one::<String>("smiles-list")
        .ok_or(eyre::eyre!("Failed to extract smiles list"))?
        .split(',')
        .collect::<Vec<_>>();

    let (storage_dir, index_name) = split_path(index_path)?;
    let index_manager = IndexManager::new(storage_dir.deref(), false)?;
    let index = index_manager.open(index_name.deref())?;
    let mut deleter = index.writer(16 * 1024 * 1024)?;
    let query_parser = QueryParser::for_index(&index, vec![]);

    for smiles in smiles_list {
        let parsed_query = create_delete_query(smiles, &query_parser);

        match parsed_query {
            Ok(parsed_query) => {
                let delete_operation = deleter.delete_query(parsed_query);
                match delete_operation {
                    Ok(_) => println!("Deleting \"{}\"", smiles),
                    Err(e) => println!("Failed to delete \"{}\": {}", smiles, e),
                }
            }
            Err(e) => println!("Failed to construct delete query for \"{}\": {}", smiles, e),
        }
    }

    let _ = deleter.commit();

    Ok(())
}

fn create_delete_query(smiles: &str, query_parser: &QueryParser) -> eyre::Result<Box<dyn Query>> {
    let (canon_taut, _fingerprint, descriptors) = process_cpd(smiles, false)?;

    let canon_smiles = canon_taut.as_smiles();
    let matching_scaffolds = scaffold_search(&canon_taut, &PARSED_SCAFFOLDS);
    let matching_scaffolds = match matching_scaffolds {
        Ok(matching_scaffolds) => Some(matching_scaffolds),
        Err(_) => None,
    };

    let raw_query =
        crate::search::identity_search::build_query(&descriptors, "", &matching_scaffolds);
    let query = format!("{raw_query} AND smiles:\"{canon_smiles}\"");
    let parsed_query = query_parser.parse_query(&query)?;
    Ok(parsed_query)
}
