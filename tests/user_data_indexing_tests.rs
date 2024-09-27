use std::collections::HashSet;

use tantivy::schema::{JsonObjectOptions, STORED, TEXT};

#[test]
fn test_json_data() {
    let mut schema_builder = tantivy::schema::SchemaBuilder::new();
    let json_options: JsonObjectOptions =
        JsonObjectOptions::from(TEXT | STORED).set_expand_dots_enabled();
    let extra_data = schema_builder.add_json_field("extra_data", json_options);
    let schema = schema_builder.build();

    let index = tantivy::IndexBuilder::new()
        .schema(schema)
        .create_in_ram()
        .unwrap();
    let mut writer = index.writer(15_000_000).unwrap();
    let doc_1 = tantivy::doc!(
        extra_data => serde_json::from_str::<serde_json::Value>(r#"{"org_ids":[1,2,3]}"#).unwrap()
    );
    writer.add_document(doc_1.clone()).unwrap();
    let doc_2 = tantivy::doc!(
        extra_data => serde_json::from_str::<serde_json::Value>(r#"{"org_ids":[3,4,5]}"#).unwrap()
    );
    writer.add_document(doc_2.clone()).unwrap();
    writer.commit().unwrap();

    let searcher = index.reader().unwrap().searcher();
    let query_parser = tantivy::query::QueryParser::for_index(&index, vec![extra_data]);
    let collector = tantivy::collector::TopDocs::with_limit(100);

    let query = query_parser.parse_query("extra_data.org_ids:2").unwrap();
    let results = searcher.search(&query, &collector).unwrap();
    assert_eq!(results.len(), 1);
    let retrieved_doc = searcher
        .doc::<tantivy::TantivyDocument>(results[0].1)
        .unwrap();
    assert_eq!(retrieved_doc, doc_1);

    let query = query_parser.parse_query("extra_data.org_ids:5").unwrap();
    let results = searcher.search(&query, &collector).unwrap();
    assert_eq!(results.len(), 1);
    let retrieved_doc = searcher
        .doc::<tantivy::TantivyDocument>(results[0].1)
        .unwrap();
    assert_eq!(retrieved_doc, doc_2);

    let query = query_parser.parse_query("extra_data.org_ids:3").unwrap();
    let results = searcher.search(&query, &collector).unwrap();
    assert_eq!(results.len(), 2);

    let set = results
        .into_iter()
        .map(|(_, doc_id)| {
            serde_json::to_string(&searcher.doc::<tantivy::TantivyDocument>(doc_id).unwrap())
                .unwrap()
        })
        .collect::<HashSet<_>>();

    assert!(set.contains(&serde_json::to_string(&doc_1).unwrap()));
    assert!(set.contains(&serde_json::to_string(&doc_2).unwrap()));
}
