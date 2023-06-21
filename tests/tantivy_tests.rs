// use super::prelude::*;
use rdkit::{MolBlockIter, ROMol, RWMol};
use serde_json::{Map, Value};
use std::collections::HashMap;
use tantivy::schema::Field;
use cheminee::tantivy::*;

#[test]
fn range_search_test() {
    let index_schema = schema();
    println!("{:?}", index_schema);
    println!("{:?}", index_schema.get_field("smile").unwrap());
}
