use crate::command_line::{indexing::split_path, prelude::*};
use crate::indexing::index_manager::IndexManager;
use crate::search::identity_search::identity_search;
use crate::search::prepare_query_structure;
use crate::search::scaffold_search::{scaffold_search, PARSED_SCAFFOLDS};
use std::ops::Deref;
use tantivy::query::QueryParser;

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
        .arg(
            Arg::new("use_scaffolds")
                .required(false)
                .long("use_scaffolds")
                .short('u')
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
    let use_scaffolds = matches.get_one::<String>("use_scaffolds");

    // by default, we will use scaffold-based indexing
    let use_scaffolds = if let Some(use_scaffolds) = use_scaffolds {
        matches!(use_scaffolds.as_str(), "true")
    } else {
        true
    };

    let (storage_dir, index_name) = split_path(index_path)?;
    let index_manager = IndexManager::new(storage_dir.deref(), false)?;
    let index = index_manager.open(index_name.deref())?;
    let mut deleter = index.writer(16 * 1024 * 1024)?;
    let reader = index.reader()?;
    let searcher = reader.searcher();
    let query_parser = QueryParser::for_index(&index, vec![]);

    let scaffolds = if use_scaffolds {
        &PARSED_SCAFFOLDS
    } else {
        &Vec::new()
    };

    for smiles in smiles_list {
        let attributes = prepare_query_structure(smiles);

        if let Ok((canon_taut, fingerprint, descriptors)) = attributes {
            let canon_smiles = canon_taut.as_smiles();

            let matching_scaffolds = if !scaffolds.is_empty() {
                scaffold_search(&canon_taut, scaffolds)?
            } else {
                Vec::new()
            };

            let result = identity_search(
                &searcher,
                &canon_taut,
                &matching_scaffolds,
                fingerprint.0.as_bitslice(),
                &descriptors,
                "",
            );

            if let Ok(Some(_)) = result {
                let raw_query = crate::search::identity_search::build_query(
                    &descriptors,
                    "",
                    &matching_scaffolds,
                );
                let query = format!("{raw_query} AND smiles:\"{canon_smiles}\"");

                let query = query_parser.parse_query(&query);

                if query.is_ok() {
                    let query_result = deleter.delete_query(query.unwrap());
                    if query_result.is_ok() {
                        let opstamp = deleter.commit();
                        if opstamp.is_ok() {
                            println!("Deleting \"{}\"", canon_smiles);
                        }
                    }
                }
            } else {
                println!("Entry {:?} was not found in the database", canon_smiles);
            }
        } else {
            println!(
                "Invalid smiles detected for {:?}: {:?}",
                smiles,
                attributes.err().unwrap()
            )
        }
    }

    Ok(())
}
