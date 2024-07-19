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
        crate::search::identity_search::build_identity_query(&descriptors, "", &matching_scaffolds);
    let query = format!("{raw_query} AND smiles:\"{canon_smiles}\"");
    let parsed_query = query_parser.parse_query(&query)?;
    Ok(parsed_query)
}

#[cfg(test)]
mod tests {
    use crate::command_line::indexing::bulk_delete::create_delete_query;
    use crate::search::compound_processing::process_cpd;
    use serde_json::json;
    use tantivy::query::QueryParser;
    use tantivy::schema::{JsonObjectOptions, SchemaBuilder, FAST, INDEXED, STORED, STRING, TEXT};
    use tantivy::{doc, IndexBuilder};

    #[test]
    fn test_create_delete_query() {
        let test_smiles = "C1=CC=CC=C1C(C)C";
        let (canon_taut, fingerprint, descriptors) = process_cpd(test_smiles, false).unwrap();

        let mut builder = SchemaBuilder::new();

        let smiles_field = builder.add_text_field("smiles", STRING | STORED);
        let fingerprint_field = builder.add_bytes_field("fingerprint", FAST | STORED);

        let mut doc = doc!(
            smiles_field => canon_taut.as_smiles(),
            fingerprint_field => fingerprint.0.clone().into_vec()
        );

        for (descriptor, val) in &descriptors {
            if descriptor.starts_with("Num") || descriptor.starts_with("lipinski") {
                let current_field = builder.add_i64_field(descriptor, INDEXED | STORED);

                doc.add_field_value(current_field, *val as i64);
            } else {
                let current_field = builder.add_f64_field(descriptor, FAST | STORED);

                doc.add_field_value(current_field, *val);
            }
        }

        let json_options: JsonObjectOptions =
            JsonObjectOptions::from(TEXT | STORED).set_expand_dots_enabled();

        let extra_data_field = builder.add_json_field("extra_data", json_options);

        doc.add_field_value(extra_data_field, json!({"extra_data": ""}));

        let schema = builder.build();
        let builder = IndexBuilder::new().schema(schema);
        let index = builder.create_in_ram().unwrap();

        let mut index_writer = index.writer_with_num_threads(1, 50 * 1024 * 1024).unwrap();

        index_writer.add_document(doc).unwrap();
        index_writer.commit().unwrap();

        let query_parser = QueryParser::for_index(&index, vec![]);

        let parsed_query = create_delete_query(test_smiles, &query_parser);
        assert!(parsed_query.is_ok());
    }
}
