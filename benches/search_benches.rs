#![feature(test)]

extern crate test;

use cheminee::command_line::indexing::bulk_index::create_tantivy_doc;
use cheminee::command_line::indexing::split_path;
use cheminee::indexing::index_manager::IndexManager;
use cheminee::indexing::{open_index, KNOWN_DESCRIPTORS};
use cheminee::schema::LIBRARY;
use cheminee::search::basic_search::basic_search;
use cheminee::search::compound_processing::standardize_smiles;
use cheminee::search::identity_search::identity_search;
use cheminee::search::scaffold_search::{scaffold_search, PARSED_SCAFFOLDS};
use cheminee::search::structure_search::structure_search;
use cheminee::search::{aggregate_query_hits, prepare_query_structure, StructureSearchHit};
use rayon::prelude::*;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use tantivy::schema::Field;
use test::Bencher;

#[bench]
fn bench_basic_search_success(b: &mut Bencher) {
    let rep_smiles =
        "C[S+](CC[C@@H](C(=O)[O-])[NH3+])C[C@@H]1[C@H]([C@H]([C@@H](O1)N2C=NC3=C(N=CN=C32)N)O)O";
    let query = "NumRings:[0 TO 20]".to_string();
    let index_path = "/tmp/cheminee/benchmark_index";
    let _index_name = build_test_index(index_path, rep_smiles);

    b.iter(|| {
        let index = open_index(index_path).unwrap();
        let reader = index.reader().unwrap();
        let searcher = reader.searcher();
        let results = basic_search(&searcher, &query, 100).unwrap();
        let _final_results = aggregate_query_hits(searcher, results, &query).unwrap();
    });
}

#[bench]
fn bench_basic_search_failure(b: &mut Bencher) {
    let rep_smiles =
        "C[S+](CC[C@@H](C(=O)[O-])[NH3+])C[C@@H]1[C@H]([C@H]([C@@H](O1)N2C=NC3=C(N=CN=C32)N)O)O";
    let query = "NumRings:[20 TO 40]".to_string();
    let index_path = "/tmp/cheminee/benchmark_index";
    let _index_name = build_test_index(index_path, rep_smiles);

    b.iter(|| {
        let index = open_index(index_path).unwrap();
        let reader = index.reader().unwrap();
        let searcher = reader.searcher();
        let results = basic_search(&searcher, &query, 100).unwrap();
        let _final_results = aggregate_query_hits(searcher, results, &query).unwrap();
    });
}

#[bench]
fn bench_identity_search(b: &mut Bencher) {
    let rep_smiles =
        "C[S+](CC[C@@H](C(=O)[O-])[NH3+])C[C@@H]1[C@H]([C@H]([C@@H](O1)N2C=NC3=C(N=CN=C32)N)O)O";
    let index_path = "/tmp/cheminee/benchmark_index";
    let index_name = build_test_index(index_path, rep_smiles);

    b.iter(|| {
        let index = open_index(index_path).unwrap();
        let reader = index.reader().unwrap();
        let searcher = reader.searcher();

        let (query_canon_taut, fingerprint, descriptors) =
            prepare_query_structure(rep_smiles).unwrap();

        let matching_scaffolds =
            scaffold_search(&fingerprint.0, &query_canon_taut, &PARSED_SCAFFOLDS).unwrap();

        let results = identity_search(
            &searcher,
            &query_canon_taut,
            &Some(matching_scaffolds),
            fingerprint.0.as_bitslice(),
            &descriptors,
            false,
            "",
        )
        .unwrap();

        package_results(results, rep_smiles, false);
    });

    let index_manager = IndexManager::new("/tmp/cheminee", false).unwrap();
    let _ = index_manager.delete(index_name.deref());
}

#[bench]
fn bench_substructure_search(b: &mut Bencher) {
    let rep_smiles =
        "C[S+](CC[C@@H](C(=O)[O-])[NH3+])C[C@@H]1[C@H]([C@H]([C@@H](O1)N2C=NC3=C(N=CN=C32)N)O)O";
    let query_smiles = "C[C@@H]1[C@H]([C@H]([C@@H](O1)[N]2C=NC3=C(N=CN=C23)N)O)O";
    let index_path = "/tmp/cheminee/benchmark_index";
    let index_name = build_test_index(index_path, rep_smiles);

    b.iter(|| {
        let index = open_index(index_path).unwrap();
        let reader = index.reader().unwrap();
        let searcher = reader.searcher();

        let query_canon_taut = standardize_smiles(query_smiles, false).unwrap();

        let results = structure_search(
            &searcher,
            &query_canon_taut,
            "substructure",
            true,
            100,
            false,
            "",
        )
        .unwrap();

        package_results(results, query_smiles, false);
    });

    let index_manager = IndexManager::new("/tmp/cheminee", false).unwrap();
    let _ = index_manager.delete(index_name.deref());
}

#[bench]
fn bench_superstructure_search(b: &mut Bencher) {
    let rep_smiles = "C[C@@H]1[C@H]([C@H]([C@@H](O1)[N]2C=NC3=C(N=CN=C23)N)O)O";
    let query_smiles =
        "C[S+](CC[C@@H](C(=O)[O-])[NH3+])C[C@@H]1[C@H]([C@H]([C@@H](O1)N2C=NC3=C(N=CN=C32)N)O)O";
    let index_path = "/tmp/cheminee/benchmark_index";
    let index_name = build_test_index(index_path, rep_smiles);

    b.iter(|| {
        let index = open_index(index_path).unwrap();
        let reader = index.reader().unwrap();
        let searcher = reader.searcher();

        let query_canon_taut = standardize_smiles(query_smiles, false).unwrap();

        let results = structure_search(
            &searcher,
            &query_canon_taut,
            "superstructure",
            true,
            100,
            false,
            "",
        )
        .unwrap();

        package_results(results, query_smiles, false);
    });

    let index_manager = IndexManager::new("/tmp/cheminee", false).unwrap();
    let _ = index_manager.delete(index_name.deref());
}

fn build_test_index(index_path: &str, rep_smiles: &str) -> String {
    let (storage_dir, index_name) = split_path(&index_path.to_string()).unwrap();
    let index_manager = IndexManager::new(storage_dir.deref(), true).unwrap();
    let schema = LIBRARY.get("descriptor_v1").unwrap().clone();

    if index_manager.exists(index_name.deref()).unwrap().is_some() {
        return index_name;
    }

    let index = index_manager
        .create(index_name.deref(), &schema, true, Some("exactmw"))
        .unwrap();
    let mut writer = index.writer(16 * 1024 * 1024).unwrap();

    let smiles_field = schema.get_field("smiles").unwrap();
    let fingerprint_field = schema.get_field("fingerprint").unwrap();
    let extra_data_field = schema.get_field("extra_data").unwrap();
    let descriptor_fields = KNOWN_DESCRIPTORS
        .iter()
        .map(|kd| (*kd, schema.get_field(kd).unwrap()))
        .collect::<HashMap<&str, Field>>();

    let mut map = serde_json::Map::new();
    map.insert("smiles".to_string(), Value::String(rep_smiles.to_string()));
    let record: Value = Value::Object(map);

    let _ = (0..100)
        .into_par_iter()
        .map(|_i| {
            let doc1 = create_tantivy_doc(
                record.clone(),
                smiles_field,
                fingerprint_field,
                &descriptor_fields,
                extra_data_field,
            )
            .unwrap();

            let _ = writer.add_document(doc1).unwrap();
        })
        .collect::<Vec<_>>();

    let _ = writer.commit();

    index_name
}

fn package_results(results: HashSet<(String, String)>, query_smiles: &str, used_tautomers: bool) {
    let _final_results = results
        .into_par_iter()
        .map(|(smiles, extra_data)| StructureSearchHit {
            extra_data,
            smiles,
            score: 1.0,
            query: query_smiles.into(),
            used_tautomers,
        })
        .collect::<Vec<_>>();
}
