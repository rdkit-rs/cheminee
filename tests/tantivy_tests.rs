use cheminee::tantivy::*;

#[test]
fn range_search_test() {
    let index_schema = schema();
    println!("{:?}", index_schema);
    println!("{:?}", index_schema.get_field("smile").unwrap());
}
