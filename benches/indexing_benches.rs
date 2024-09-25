#![feature(test)]

extern crate test;

use cheminee::command_line::indexing::bulk_index::create_tantivy_doc;
use cheminee::indexing::KNOWN_DESCRIPTORS;
use cheminee::schema::LIBRARY;
use serde_json::Value;
use std::collections::HashMap;
use tantivy::schema::Field;
use test::Bencher;

#[bench]
fn bench_create_tantivy_doc(b: &mut Bencher) {
    let schema = LIBRARY.get("descriptor_v1").unwrap().clone();
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
        let _ = create_tantivy_doc(
            record.clone(),
            smiles_field,
            fingerprint_field,
            &descriptor_fields,
            extra_data_field,
        );
    });
}

// running 1 test
// test bench_create_tantivy_doc ... bench:   6,527,306.20 ns/iter (+/- 562,411.21)
