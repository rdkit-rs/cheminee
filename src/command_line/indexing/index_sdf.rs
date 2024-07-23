use std::collections::HashMap;

use rdkit::{MolBlockIter, ROMol, RWMol};
use serde_json::{Map, Value};
use tantivy::schema::Field;

use crate::command_line::prelude::*;
use crate::search::compound_processing::process_cpd;
use crate::search::scaffold_search::{scaffold_search, PARSED_SCAFFOLDS};

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

pub fn action(matches: &ArgMatches) -> eyre::Result<usize> {
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
    let descriptors_fields = KNOWN_DESCRIPTORS
        .iter()
        .map(|kd| (*kd, schema.get_field(kd).unwrap()))
        .collect::<HashMap<&str, Field>>();

    let mut counter = 0;
    for mol in mol_iter {
        if mol.is_err() {
            continue;
        }

        let mol = mol.unwrap();
        let mol: ROMol = mol.to_ro_mol();

        // By default, do not attempt to fix problematic molecules
        let process_result = process_cpd(mol.as_smiles().as_str(), false);
        if process_result.is_err() {
            continue;
        }

        let (canon_taut, fp, computed) = process_result.unwrap();

        let json: serde_json::Value = serde_json::to_value(&computed)?;
        let descriptors_map: Map<String, Value> = if let serde_json::Value::Object(map) = json {
            map
        } else {
            panic!("not an object");
        };

        let mut doc = doc!(
            smiles_field => canon_taut.as_smiles(),
            fingerprint_field => fp.0.into_vec()
        );

        for field in KNOWN_DESCRIPTORS {
            if let Some(serde_json::Value::Number(val)) = descriptors_map.get(field) {
                let current_field = *descriptors_fields
                    .get(field)
                    .ok_or(eyre::eyre!("Failed to extract field"))?;
                let current_value = val
                    .as_f64()
                    .ok_or(eyre::eyre!("Failed to convert descriptor to float"))?;
                if field.starts_with("Num") || field.starts_with("lipinski") {
                    doc.add_field_value(current_field, current_value as i64);
                } else {
                    doc.add_field_value(current_field, current_value);
                };
            }
        }

        let scaffold_matches = scaffold_search(&canon_taut, &PARSED_SCAFFOLDS)?;

        let scaffold_json = match scaffold_matches.is_empty() {
            true => serde_json::json!({"scaffolds": vec![-1]}),
            false => serde_json::json!({"scaffolds": scaffold_matches}),
        };

        doc.add_field_value(extra_data_field, scaffold_json);

        index_writer.add_document(doc)?;

        if counter > 0 && counter % 10_000 == 0 {
            index_writer.commit()?;
            println!("{:?} compounds written so far", counter);
        }

        counter += 1;
    }

    index_writer.commit()?;
    Ok(counter)
}
