use crate::command_line::indexing::split_path;
use crate::command_line::prelude::*;
use crate::indexing::index_manager::IndexManager;
use crate::search::compound_processing::process_cpd;
use bitvec::macros::internal::funty::Fundamental;
use rayon::prelude::*;
use serde_json::Value;
use std::{collections::HashMap, fs::File, io::BufReader, ops::Deref};
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

    let smile_field = schema.get_field("smile")?;
    let fingerprint_field = schema.get_field("fingerprint")?;
    let extra_data_field = schema.get_field("extra_data")?;
    let descriptor_fields = KNOWN_DESCRIPTORS
        .iter()
        .map(|kd| (*kd, schema.get_field(kd).unwrap()))
        .collect::<HashMap<&str, Field>>();

    let file = File::open(json_path)?;
    let reader = BufReader::new(file);
    let json_array: Vec<Value> = serde_json::from_reader(reader)?;

    let _ = json_array
        .into_par_iter()
        .map(|ob| {
            let smile = ob
                .get("smile")
                .ok_or(eyre::eyre!("Failed to extract smiles"))
                .unwrap()
                .to_string()
                // For some reason there are leading and trailing quotations marks messing up the parsing...
                .replace("\"", "");

            let extra_data = ob.get("extra_data");

            let doc = create_tantivy_doc(
                smile.deref(),
                extra_data.cloned(),
                smile_field,
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
                            println!("Failed to index smiles {}: {}", smile, e);
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to index smiles {}: {}", smile, e);
                }
            }
        })
        .collect::<Vec<_>>();

    let _commit = writer.commit()?;

    Ok(())
}

fn create_tantivy_doc(
    smile: &str,
    extra_data: Option<Value>,
    smile_field: Field,
    fingerprint_field: Field,
    descriptor_fields: &HashMap<&str, Field>,
    extra_data_field: Field,
) -> eyre::Result<tantivy::Document> {
    let (canon_taut, fingerprint, descriptors) = process_cpd(smile)?;

    let mut doc = doc!(
        smile_field => canon_taut.as_smile(),
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
