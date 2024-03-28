use crate::command_line::{indexing::split_path, prelude::*};
use crate::indexing::index_manager::IndexManager;
use crate::search::compound_processing::process_cpd;
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
                .required(false)
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
    let mut writer = index.writer(8 * 1024 * 1024)?;
    let schema = index.schema();

    let smiles_field = schema.get_field("smiles")?;
    let fingerprint_field = schema.get_field("fingerprint")?;
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
        let record: serde_json::Value = serde_json::from_str(&line)?;

        record_vec.push(record);
        if record_vec.len() == chunksize {
            let _ = record_vec
                .clone()
                .into_par_iter()
                .map(|r| {
                    let doc = create_tantivy_doc(
                        r,
                        smiles_field,
                        fingerprint_field,
                        &descriptor_fields,
                        extra_data_field,
                    );

                    match doc {
                        Ok(doc) => {
                            let write_operation = writer.add_document(doc);

                            match write_operation {
                                Ok(_) => (),
                                Err(e) => {
                                    println!("Failed doc creation: {:?}", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("Failed doc creation: {:?}", e);
                        }
                    }

                    ()
                })
                .collect::<Vec<_>>();

            let _ = writer.commit();

            record_vec.clear();
        }
    }

    Ok(())
}

fn create_tantivy_doc(
    record: serde_json::Value,
    smiles_field: Field,
    fingerprint_field: Field,
    descriptor_fields: &HashMap<&str, Field>,
    extra_data_field: Field,
) -> eyre::Result<tantivy::Document> {
    let smiles = record
        .get("smiles")
        .ok_or(eyre::eyre!("Failed to extract smiles"))?
        .as_str()
        .ok_or(eyre::eyre!("Failed to convert smiles to str"))?;
    let extra_data = record.get("extra_data").cloned();

    // By default, do not attempt to fix problematic molecules
    let (canon_taut, fingerprint, descriptors) = process_cpd(smiles, false)?;

    let mut doc = doc!(
        smiles_field => canon_taut.as_smiles(),
        fingerprint_field => fingerprint.0.into_vec()
    );

    if let Some(extra_data) = extra_data {
        doc.add_field_value(extra_data_field, extra_data);
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
