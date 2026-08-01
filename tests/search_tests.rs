use bitvec::store::BitStore;
use cheminee::search::compound_processing::{process_cpd, standardize_smiles};
use cheminee::search::identity_search::{build_identity_query, identity_search};
use cheminee::search::scaffold_network::{
    scaffold_network_for_smiles, scaffold_networks_for_smiles, ScaffoldNetworkOptions,
    ScaffoldNetworkResult,
};
use cheminee::search::scaffold_search::{scaffold_search, PARSED_SCAFFOLDS};
use cheminee::search::similarity_search::build_similarity_query;
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
fn test_build_similarity_query() {
    let ranked_clusters = vec![0, 2];
    let extra_query = "NumAtoms:[1 TO 100] AND NumAmideBonds: [1 TO 5]";
    let query = build_similarity_query(&ranked_clusters, extra_query, 0.1).unwrap();
    assert_eq!(query, "(other_descriptors.similarity_cluster:0 OR other_descriptors.similarity_cluster:2) AND NumAtoms:[1 TO 100] AND NumAmideBonds: [1 TO 5]");
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

/// 2-phenylquinoline: one fused bicycle plus a pendant ring, so the hierarchy has
/// something in it.
const PHENYLQUINOLINE: &str = "c1ccc(-c2ccc3ncccc3c2)cc1";

fn scaffold_smiles(result: &ScaffoldNetworkResult) -> Vec<&str> {
    result
        .nodes
        .iter()
        .map(|n| n.scaffold_smiles.as_str())
        .collect()
}

fn no_generics() -> ScaffoldNetworkOptions {
    ScaffoldNetworkOptions {
        include_generic_scaffolds: Some(false),
        include_generic_bond_scaffolds: Some(false),
        ..Default::default()
    }
}

#[test]
fn test_scaffold_network_hierarchy() {
    let result = scaffold_network_for_smiles(PHENYLQUINOLINE, &no_generics()).unwrap();

    assert_eq!(result.smiles, PHENYLQUINOLINE);
    assert_eq!(result.standardized_smiles.as_deref(), Some(PHENYLQUINOLINE));
    assert_eq!(result.error, None);
    assert_eq!(
        scaffold_smiles(&result),
        [
            PHENYLQUINOLINE,
            "*c1ccccc1",
            "c1ccccc1",
            "*c1ccc2ncccc2c1",
            "c1ccc2ncccc2c1",
        ]
    );

    // every edge points from the more specific scaffold up to the more general one, and
    // carries both SMILES so an edge can be consumed without the node list
    let hierarchy = result
        .edges
        .iter()
        .map(|e| (e.child_smiles.as_str(), e.parent_smiles.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        hierarchy,
        [
            (PHENYLQUINOLINE, "*c1ccccc1"),
            ("*c1ccccc1", "c1ccccc1"),
            (PHENYLQUINOLINE, "*c1ccc2ncccc2c1"),
            ("*c1ccc2ncccc2c1", "c1ccc2ncccc2c1"),
        ]
    );

    // and the indices agree with the SMILES they were resolved from
    for edge in &result.edges {
        assert_eq!(
            result.nodes[edge.parent_idx as usize].scaffold_smiles,
            edge.parent_smiles
        );
        assert_eq!(
            result.nodes[edge.child_idx as usize].scaffold_smiles,
            edge.child_smiles
        );
    }
}

#[test]
fn test_scaffold_network_single_ring_is_trivial() {
    let result = scaffold_network_for_smiles("c1ccccc1", &no_generics()).unwrap();

    // nothing to strip: benzene is already its own scaffold
    assert_eq!(scaffold_smiles(&result), ["c1ccccc1"]);
    assert!(result.edges.is_empty());
    assert_eq!(result.nodes[0].count, 1);
    assert_eq!(result.nodes[0].mol_count, Some(1));
}

#[test]
fn test_scaffold_network_generic_and_attachment_flags() {
    let result = scaffold_network_for_smiles(PHENYLQUINOLINE, &Default::default()).unwrap();

    let flags = result
        .nodes
        .iter()
        .map(|n| (n.scaffold_smiles.as_str(), n.is_generic, n.has_attachments))
        .collect::<Vec<_>>();

    // the generic twin of a scaffold keeps that scaffold's attachment points, which is why
    // `**1:*:*:*:*:*:1` and `*1:*:*:*:*:*:1` are both generic but differ on attachments
    assert!(flags.contains(&("c1ccccc1", false, false)));
    assert!(flags.contains(&("*c1ccccc1", false, true)));
    assert!(flags.contains(&("*1:*:*:*:*:*:1", true, false)));
    assert!(flags.contains(&("**1:*:*:*:*:*:1", true, true)));
}

#[test]
fn test_scaffold_network_params_toggle_changes_output() {
    let with_generics = scaffold_network_for_smiles(PHENYLQUINOLINE, &Default::default()).unwrap();
    let without_generics = scaffold_network_for_smiles(PHENYLQUINOLINE, &no_generics()).unwrap();

    assert_eq!(with_generics.nodes.len(), 9);
    assert_eq!(without_generics.nodes.len(), 5);
    assert!(with_generics.nodes.iter().any(|n| n.is_generic));
    assert!(!without_generics.nodes.iter().any(|n| n.is_generic));
}

#[test]
fn test_scaffold_network_empty_bond_breakers_never_fragment() {
    let options = ScaffoldNetworkOptions {
        bond_breaker_smarts: Some(vec![]),
        ..Default::default()
    };
    let result = scaffold_network_for_smiles(PHENYLQUINOLINE, &options).unwrap();

    assert_eq!(scaffold_smiles(&result), [PHENYLQUINOLINE]);
}

/// A molecule with no rings reduces to the empty scaffold, whose generic form is also
/// empty, so RDKit folds them into one node with an edge pointing at itself. Anything
/// walking these edges as a DAG needs to expect that.
#[test]
fn test_scaffold_network_without_rings_self_loops() {
    let result = scaffold_network_for_smiles("CCO", &Default::default()).unwrap();

    assert_eq!(scaffold_smiles(&result), ["CCO", ""]);
    let self_loops = result
        .edges
        .iter()
        .filter(|e| e.parent_idx == e.child_idx)
        .count();
    assert_eq!(self_loops, 1);
}

#[test]
fn test_scaffold_network_rejects_oversized_molecule() {
    let options = ScaffoldNetworkOptions {
        max_atoms: Some(5),
        ..Default::default()
    };

    let error = scaffold_network_for_smiles(PHENYLQUINOLINE, &options)
        .unwrap_err()
        .to_string();
    assert_eq!(error, "molecule has 16 atoms, more than the 5 allowed");
}

#[test]
fn test_scaffold_network_rejects_oversized_network() {
    let options = ScaffoldNetworkOptions {
        max_nodes: Some(2),
        ..Default::default()
    };

    let error = scaffold_network_for_smiles(PHENYLQUINOLINE, &options)
        .unwrap_err()
        .to_string();
    assert_eq!(
        error,
        "scaffold network has 9 nodes, more than the 2 allowed"
    );
}

#[test]
fn test_scaffold_network_skips_standardization_on_request() {
    let options = ScaffoldNetworkOptions {
        standardize: Some(false),
        ..Default::default()
    };

    // still canonicalized on the way out, just not run through fragment_parent and
    // tautomer canonicalization first
    let result = scaffold_network_for_smiles("C1=CC=CC=C1", &options).unwrap();
    assert_eq!(result.standardized_smiles.as_deref(), Some("c1ccccc1"));
}

#[test]
fn test_scaffold_networks_for_smiles_isolates_failures() {
    let batch = [
        "c1ccccc1".to_string(),
        "not-a-smiles".to_string(),
        PHENYLQUINOLINE.to_string(),
    ];
    let results = scaffold_networks_for_smiles(&batch, &no_generics());

    // results line up with inputs, and the bad one does not take the batch down with it
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].error, None);
    assert_eq!(results[0].nodes.len(), 1);
    assert_eq!(
        results[1].error.as_deref(),
        Some("could not convert smiles to romol (nullptr)")
    );
    assert!(results[1].nodes.is_empty());
    assert_eq!(results[2].error, None);
    assert_eq!(results[2].nodes.len(), 5);
}
