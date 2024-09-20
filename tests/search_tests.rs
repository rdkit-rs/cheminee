use cheminee::search::compound_processing::{process_cpd, standardize_smiles};
use cheminee::search::identity_search::{build_identity_query, identity_search};
use cheminee::search::scaffold_search::{scaffold_search, PARSED_SCAFFOLDS};
use cheminee::search::structure_search::{
    build_substructure_query, build_superstructure_query, structure_search,
};
use serde_json::json;
use std::collections::HashMap;
use tantivy::schema::{JsonObjectOptions, TEXT};
use tantivy::{
    doc,
    schema::{SchemaBuilder, FAST, INDEXED, STORED, STRING},
    IndexBuilder,
};

#[test]
fn test_build_identity_query() {
    let descriptors: HashMap<_, _> = [("NumAtoms".to_string(), 10.0)].into_iter().collect();
    let query = build_identity_query(&descriptors, "", &None);
    assert_eq!(query, "NumAtoms:10");
}

#[test]
fn test_build_substructure_query() {
    let descriptors: HashMap<_, _> = [("NumAtoms".to_string(), 10.0)].into_iter().collect();
    let query = build_substructure_query(&descriptors, "", &None);
    assert_eq!(query, "NumAtoms:[10 TO 10000]");
}

#[test]
fn test_build_superstructure_query() {
    let descriptors: HashMap<_, _> = [("NumAtoms".to_string(), 10.0)].into_iter().collect();
    let query = build_superstructure_query(&descriptors, "", &Some(vec![0, 1]));
    assert_eq!(
        query,
        "NumAtoms:[0 TO 10] AND (extra_data.scaffolds:0 OR extra_data.scaffolds:1 OR extra_data.scaffolds:-1)"
    );
}

#[test]
fn test_identity_search() {
    let test_smiles = "CC";
    let (query_mol, query_fingerprint, query_descriptors) =
        process_cpd(test_smiles, false).unwrap();

    let mut builder = SchemaBuilder::new();

    let smiles_field = builder.add_text_field("smiles", STRING | STORED);
    let fingerprint_field = builder.add_bytes_field("fingerprint", FAST | STORED);

    let mut doc = doc!(
        smiles_field => test_smiles,
        fingerprint_field => query_fingerprint.0.as_raw_slice()
    );

    for (descriptor, val) in &query_descriptors {
        if descriptor.starts_with("Num") || descriptor.starts_with("lipinski") {
            let current_field = builder.add_i64_field(descriptor, INDEXED | STORED);

            doc.add_field_value(current_field, *val as i64);
        } else {
            let current_field = builder.add_f64_field(descriptor, FAST | STORED);

            doc.add_field_value(current_field, *val);
        }
    }

    let schema = builder.build();

    let builder = IndexBuilder::new().schema(schema);
    let index = builder.create_in_ram().unwrap();

    let mut index_writer = index.writer_with_num_threads(1, 50 * 1024 * 1024).unwrap();

    index_writer.add_document(doc).unwrap();
    index_writer.commit().unwrap();

    let reader = index.reader().unwrap();
    let searcher = reader.searcher();

    let extra_query = "".to_string();

    let result = identity_search(
        &searcher,
        &query_mol,
        &None,
        query_fingerprint.0.as_bitslice(),
        &query_descriptors,
        true,
        &extra_query,
    )
    .unwrap();
    assert_eq!(result.len(), 1);
}

#[test]
fn test_substructure_search() {
    let index_smiles = "C1=CC=CC=C1CC2=CC=CC=C2";
    let (index_mol, index_fingerprint, index_descriptors) =
        process_cpd(index_smiles, false).unwrap();
    let index_scaffolds =
        scaffold_search(&index_fingerprint.0, &index_mol, &PARSED_SCAFFOLDS).unwrap();

    let query_smiles = "C1=CC=CC=C1";
    let query_mol = standardize_smiles(query_smiles, false).unwrap();

    let mut builder = SchemaBuilder::new();
    let smiles_field = builder.add_text_field("smiles", STRING | STORED);
    let fingerprint_field = builder.add_bytes_field("fingerprint", FAST | STORED);

    let json_options: JsonObjectOptions =
        JsonObjectOptions::from(TEXT | STORED).set_expand_dots_enabled();
    let extra_data_field = builder.add_json_field("extra_data", json_options);

    let mut doc = doc!(
        smiles_field => index_mol.as_smiles(),
        fingerprint_field => index_fingerprint.0.as_raw_slice(),
        extra_data_field => json![{ "scaffolds": index_scaffolds }],
    );

    for (descriptor, val) in &index_descriptors {
        if descriptor.starts_with("Num") || descriptor.starts_with("lipinski") {
            let current_field = builder.add_i64_field(descriptor, INDEXED | STORED);

            doc.add_field_value(current_field, *val as i64);
        } else {
            let current_field = builder.add_f64_field(descriptor, FAST | STORED);

            doc.add_field_value(current_field, *val);
        }
    }

    let schema = builder.build();

    let builder = IndexBuilder::new().schema(schema);
    let index = builder.create_in_ram().unwrap();

    let mut index_writer = index.writer_with_num_threads(1, 50 * 1024 * 1024).unwrap();

    index_writer.add_document(doc).unwrap();
    index_writer.commit().unwrap();

    let reader = index.reader().unwrap();
    let searcher = reader.searcher();

    let extra_query = "".to_string();

    let results = structure_search(
        &searcher,
        &query_mol,
        "substructure",
        true,
        10,
        true,
        &extra_query,
    )
    .unwrap();

    assert_eq!(results.len(), 1);
}

#[test]
fn test_superstructure_search() {
    let index_smiles = "C1=CC=CC=C1";
    let (index_mol, index_fingerprint, index_descriptors) =
        process_cpd(index_smiles, false).unwrap();
    let index_scaffolds =
        scaffold_search(&index_fingerprint.0, &index_mol, &PARSED_SCAFFOLDS).unwrap();

    let query_smiles = "C1=CC=CC=C1CC2=CC=CC=C2";
    let query_mol = standardize_smiles(query_smiles, false).unwrap();

    let mut builder = SchemaBuilder::new();
    let smiles_field = builder.add_text_field("smiles", STRING | STORED);
    let fingerprint_field = builder.add_bytes_field("fingerprint", FAST | STORED);

    let json_options: JsonObjectOptions =
        JsonObjectOptions::from(TEXT | STORED).set_expand_dots_enabled();
    let extra_data_field = builder.add_json_field("extra_data", json_options);

    let mut doc = doc!(
        smiles_field => index_mol.as_smiles(),
        fingerprint_field => index_fingerprint.0.as_raw_slice(),
        extra_data_field => json![{ "scaffolds": index_scaffolds }],
    );

    for (descriptor, val) in &index_descriptors {
        if descriptor.starts_with("Num") || descriptor.starts_with("lipinski") {
            let current_field = builder.add_i64_field(descriptor, INDEXED | STORED);

            doc.add_field_value(current_field, *val as i64);
        } else {
            let current_field = builder.add_f64_field(descriptor, FAST | STORED);

            doc.add_field_value(current_field, *val);
        }
    }

    let schema = builder.build();

    let builder = IndexBuilder::new().schema(schema);
    let index = builder.create_in_ram().unwrap();

    let mut index_writer = index.writer_with_num_threads(1, 50 * 1024 * 1024).unwrap();

    index_writer.add_document(doc).unwrap();
    index_writer.commit().unwrap();

    let reader = index.reader().unwrap();
    let searcher = reader.searcher();

    let extra_query = "".to_string();

    let results = structure_search(
        &searcher,
        &query_mol,
        "superstructure-search",
        true,
        10,
        true,
        &extra_query,
    )
    .unwrap();

    assert_eq!(results.len(), 1);
}
