use bitvec::store::BitStore;
use cheminee::search::compound_processing::{process_cpd, standardize_smiles};
use cheminee::search::identity_search::{build_identity_query, identity_search};
use cheminee::search::scaffold_search::{scaffold_search, PARSED_SCAFFOLDS};
use cheminee::search::structure_search::{
    build_substructure_query, build_superstructure_query, structure_search,
};
use cheminee::search::{sort_docs, sort_results};
use serde_json::json;
use std::collections::HashMap;
use tantivy::schema::{JsonObjectOptions, TEXT};
use tantivy::{
    doc,
    schema::{SchemaBuilder, FAST, INDEXED, STORED, STRING},
    DocAddress, DocId, IndexBuilder, SegmentOrdinal,
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
        "NumAtoms:[0 TO 10] AND (other_descriptors.scaffolds:0 OR other_descriptors.scaffolds:1 OR other_descriptors.scaffolds:-1)"
    );
}

#[test]
fn test_sort_docs() {
    let doc1 = DocAddress::new(1, 0);
    let doc2 = DocAddress::new(1, 1);
    let doc3 = DocAddress::new(0, 1);

    let mut doc_vec = vec![doc1, doc2, doc3];

    sort_docs(&mut doc_vec);

    assert_eq!(doc_vec, vec![doc3, doc1, doc2]);
}

#[test]
fn test_sort_results() {
    let mut results = vec![
        (
            "CC".to_string(),
            json!({"extra":"data"}),
            SegmentOrdinal::new(1),
            DocId::new(0),
        ),
        (
            "c1ccccc1".to_string(),
            json!({"extra":"data"}),
            SegmentOrdinal::new(1),
            DocId::new(1),
        ),
        (
            "c1ccc(CCc2ccccc2)cc1".to_string(),
            json!({"extra":"data"}),
            SegmentOrdinal::new(0),
            DocId::new(1),
        ),
    ];

    sort_results(&mut results);
    assert_eq!(
        results,
        vec![
            (
                "c1ccc(CCc2ccccc2)cc1".to_string(),
                json!({"extra": "data"}),
                0,
                1
            ),
            ("CC".to_string(), json!({"extra": "data"}), 1, 0),
            ("c1ccccc1".to_string(), json!({"extra": "data"}), 1, 1)
        ]
    );
}

#[test]
fn test_identity_search() {
    let test_smiles = "CC";
    let (query_mol, query_pattern_fingerprint, query_descriptors) =
        process_cpd(test_smiles, false).unwrap();

    let mut builder = SchemaBuilder::new();

    let smiles_field = builder.add_text_field("smiles", STRING | STORED);
    let pattern_fingerprint_field = builder.add_bytes_field("pattern_fingerprint", FAST | STORED);

    let json_options: JsonObjectOptions =
        JsonObjectOptions::from(TEXT | STORED).set_expand_dots_enabled();
    let _extra_data_field = builder.add_json_field("extra_data", json_options);

    let mut doc = doc!(
        smiles_field => query_mol.as_smiles(),
        pattern_fingerprint_field => query_pattern_fingerprint.0.as_raw_slice(),
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
        query_pattern_fingerprint.0.as_bitslice(),
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
    let (index_mol, index_pattern_fingerprint, index_descriptors) =
        process_cpd(index_smiles, false).unwrap();
    let index_scaffolds =
        scaffold_search(&index_pattern_fingerprint.0, &index_mol, &PARSED_SCAFFOLDS).unwrap();

    let query_smiles = "C1=CC=CC=C1";
    let query_mol = standardize_smiles(query_smiles, false).unwrap();

    let mut builder = SchemaBuilder::new();
    let smiles_field = builder.add_text_field("smiles", STRING | STORED);
    let pattern_fingerprint_field = builder.add_bytes_field("pattern_fingerprint", FAST | STORED);

    let extra_data_options: JsonObjectOptions =
        JsonObjectOptions::from(TEXT | STORED).set_expand_dots_enabled();
    let _extra_data_field = builder.add_json_field("extra_data", extra_data_options);

    let other_descriptors_options: JsonObjectOptions =
        JsonObjectOptions::from(TEXT).set_expand_dots_enabled();
    let other_descriptors_field =
        builder.add_json_field("other_descriptors", other_descriptors_options);

    let mut doc = doc!(
        smiles_field => index_mol.as_smiles(),
        pattern_fingerprint_field => index_pattern_fingerprint.0.as_raw_slice(),
        other_descriptors_field => json![{ "scaffolds": index_scaffolds }],
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
    let (index_mol, index_pattern_fingerprint, index_descriptors) =
        process_cpd(index_smiles, false).unwrap();
    let index_scaffolds =
        scaffold_search(&index_pattern_fingerprint.0, &index_mol, &PARSED_SCAFFOLDS).unwrap();

    let query_smiles = "C1=CC=CC=C1CC2=CC=CC=C2";
    let query_mol = standardize_smiles(query_smiles, false).unwrap();

    let mut builder = SchemaBuilder::new();
    let smiles_field = builder.add_text_field("smiles", STRING | STORED);
    let pattern_fingerprint_field = builder.add_bytes_field("pattern_fingerprint", FAST | STORED);

    let extra_data_options: JsonObjectOptions =
        JsonObjectOptions::from(TEXT | STORED).set_expand_dots_enabled();
    let _extra_data_field = builder.add_json_field("extra_data", extra_data_options);

    let other_descriptors_options: JsonObjectOptions =
        JsonObjectOptions::from(TEXT).set_expand_dots_enabled();
    let other_descriptors_field =
        builder.add_json_field("other_descriptors", other_descriptors_options);

    let mut doc = doc!(
        smiles_field => index_mol.as_smiles(),
        pattern_fingerprint_field => index_pattern_fingerprint.0.as_raw_slice(),
        other_descriptors_field => json![{ "scaffolds": index_scaffolds }],
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
