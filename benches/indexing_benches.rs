#![feature(test)]

extern crate test;

use cheminee::command_line::indexing::bulk_index::create_tantivy_doc;
use cheminee::command_line::indexing::split_path;
use cheminee::indexing::index_manager::IndexManager;
use cheminee::indexing::KNOWN_DESCRIPTORS;
use cheminee::schema::LIBRARY;
use serde_json::Value;
use std::collections::HashMap;
use std::ops::Deref;
use tantivy::schema::Field;
use test::Bencher;

#[bench]
fn bench_bulk_index(b: &mut Bencher) {
    let (storage_dir, index_name) = split_path(&"/tmp/test/benchmark_index".to_string()).unwrap();
    let index_manager = IndexManager::new(storage_dir.deref(), true).unwrap();
    let schema = LIBRARY.get("descriptor_v1").unwrap().clone();
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

    let smi =
        "C[S+](CC[C@@H](C(=O)[O-])[NH3+])C[C@@H]1[C@H]([C@H]([C@@H](O1)N2C=NC3=C(N=CN=C32)N)O)O";
    let mut map = serde_json::Map::new();
    map.insert("smiles".to_string(), Value::String(smi.to_string()));

    let record: Value = Value::Object(map);

    b.iter(|| {
        let doc = create_tantivy_doc(
            record.clone(),
            smiles_field,
            fingerprint_field,
            &descriptor_fields,
            extra_data_field,
        )
        .unwrap();

        let _ = writer.add_document(doc).unwrap();
    });

    let _ = writer.commit();
    let _ = index_manager.delete(index_name.deref());
}
