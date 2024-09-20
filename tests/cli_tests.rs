use cheminee::command_line::indexing::bulk_delete::create_delete_query;
use cheminee::indexing::index_manager::IndexManager;
use cheminee::schema::LIBRARY;
use cheminee::search::compound_processing::process_cpd;
use serde_json::json;
use tantivy::query::QueryParser;
use tantivy::schema::{JsonObjectOptions, SchemaBuilder, FAST, INDEXED, STORED, STRING, TEXT};
use tantivy::{doc, IndexBuilder};

#[test]
fn index_manager() -> eyre::Result<()> {
    let index_manager = IndexManager::new("/tmp/xavier", true)?;

    let schema = LIBRARY.get("descriptor_v1").unwrap();

    let _index = index_manager.create("structure-search", schema, true, Some("exactmw"))?;

    let _index = index_manager.open("structure-search")?;

    assert!(index_manager.exists("structure-search").unwrap().is_some());

    let index_paths = index_manager.list()?;
    assert_eq!(index_paths[0], "structure-search");

    let _ = index_manager.delete("structure-search");
    assert!(index_manager.exists("structure-search").unwrap().is_none());

    Ok(())
}

#[test]
fn test_create_delete_query() {
    let test_smiles = "C1=CC=CC=C1C(C)C";
    let (canon_taut, fingerprint, descriptors) = process_cpd(test_smiles, false).unwrap();

    let mut builder = SchemaBuilder::new();

    let smiles_field = builder.add_text_field("smiles", STRING | STORED);
    let fingerprint_field = builder.add_bytes_field("fingerprint", FAST | STORED);

    let mut doc = doc!(
        smiles_field => canon_taut.as_smiles(),
        fingerprint_field => fingerprint.0.as_raw_slice()
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
