use crate::command_line::{indexing::split_path, prelude::*};
use crate::indexing::index_manager::IndexManager;
use crate::search::compound_processing::process_cpd;
use crate::search::scaffold_search::{scaffold_search, PARSED_SCAFFOLDS};
use bitvec::macros::internal::funty::Fundamental;
use rayon::prelude::*;
use std::{collections::HashMap, fs::File, io::BufRead, io::BufReader, ops::Deref};
use tantivy::schema::Field;

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
    let mut writer = index.writer(16 * 1024 * 1024)?;
    let schema = index.schema();

    let smiles_field = schema.get_field("smiles")?;
    let pattern_fingerprint_field = schema.get_field("pattern_fingerprint")?;
    let extra_data_field = schema.get_field("extra_data")?;
    let descriptor_fields = KNOWN_DESCRIPTORS
        .iter()
        .map(|kd| (*kd, schema.get_field(kd).unwrap()))
        .collect::<HashMap<&str, Field>>();

    let file = File::open(json_path)?;
    let reader = BufReader::new(file);
    let chunksize = 1000;
    let mut record_vec = Vec::with_capacity(chunksize);

    for result_line in reader.lines() {
        let line = result_line?;
        let record = serde_json::from_str(&line)?;

        record_vec.push(record);
        if record_vec.len() == chunksize {
            let _ = record_vec
                .clone()
                .into_par_iter()
                .map(|r| {
                    let doc = create_tantivy_doc(
                        r,
                        smiles_field,
                        pattern_fingerprint_field,
                        &descriptor_fields,
                        extra_data_field,
                    );

                    match doc {
                        Ok(doc) => {
                            let write_operation = writer.add_document(doc);

                            match write_operation {
                                Ok(_) => (),
                                Err(e) => {
                                    log::info!("Failed doc creation: {:?}", e);
                                }
                            }
                        }
                        Err(e) => {
                            log::info!("Failed doc creation: {:?}", e);
                        }
                    }
                })
                .collect::<Vec<_>>();

            record_vec.clear();
        }
    }

    let _ = writer.commit();

    Ok(())
}

fn create_tantivy_doc(
    record: serde_json::Value,
    smiles_field: Field,
    pattern_fingerprint_field: Field,
    descriptor_fields: &HashMap<&str, Field>,
    extra_data_field: Field,
) -> eyre::Result<impl tantivy::Document> {
    let smiles = record
        .get("smiles")
        .ok_or(eyre::eyre!("Failed to extract smiles"))?
        .as_str()
        .ok_or(eyre::eyre!("Failed to convert smiles to str"))?;
    let extra_data = record.get("extra_data").cloned();

    // By default, do not attempt to fix problematic molecules
    let (canon_taut, pattern_fingerprint, descriptors) = process_cpd(smiles, false)?;

    let mut doc = doc!(
        smiles_field => canon_taut.as_smiles(),
        pattern_fingerprint_field => pattern_fingerprint.0.as_raw_slice()
    );

    let scaffold_matches = scaffold_search(&pattern_fingerprint.0, &canon_taut, &PARSED_SCAFFOLDS)?;

    let scaffold_json = match scaffold_matches.is_empty() {
        true => serde_json::json!({"scaffolds": vec![-1]}),
        false => serde_json::json!({"scaffolds": scaffold_matches}),
    };

    let extra_data_json = combine_json_objects(Some(scaffold_json), extra_data);
    if let Some(extra_data_json) = extra_data_json {
        doc.add_field_value(extra_data_field, extra_data_json);
    }

    for field in KNOWN_DESCRIPTORS {
        if let Some(val) = descriptors.get(field) {
            if field.starts_with("Num") || field.starts_with("lipinski") {
                let int = val.as_f64() as i64;
                doc.add_field_value(*descriptor_fields.get(field).unwrap(), int);
            } else {
                doc.add_field_value(*descriptor_fields.get(field).unwrap(), val.as_f64());
            };
        }
    }

    Ok(doc)
}
