use rdkit::ROMol;
use std::collections::HashMap;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::Searcher;

const DESCRIPTOR_ALLOW_LIST: [&'static str; 2] = ["meep", "boop"];

pub fn substructure_search(searcher: &Searcher, smile: &str) -> eyre::Result<()> {
    let schema = searcher.schema();
    let index = searcher.index();

    let (_canonical_tautomer, _fingerprint, descriptors) =
        crate::analysis::compound_processing::process_cpd(smile)?;

    let query = build_query(&descriptors);

    let query_parser = QueryParser::for_index(index, vec![]);
    let parsed_query = query_parser.parse_query(&query)?;

    let top_docs = searcher.search(&parsed_query, &TopDocs::with_limit(1000))?;

    // (DocId, Smile, Fingerprint, DescriptorsHashMap)
    let smile_field = schema.get_field("smile")?;
    let fingerprint_field = schema.get_field("fingerprint")?;
    for (score, docaddr) in top_docs {
        let doc = searcher.doc(docaddr)?;
        let smile = doc.get_first(smile_field);
        let fingerprint = doc.get_first(fingerprint_field);

        println!("{:?}/{:?}", smile, fingerprint);
    }

    Ok(())
}

fn build_query(descriptors: &HashMap<String, f64>) -> String {
    let mut query_parts = Vec::with_capacity(descriptors.len());

    for (k, v) in descriptors {
        if DESCRIPTOR_ALLOW_LIST.contains(&k.as_str()) {
            query_parts.push(format!("{k}: [{v} TO 10000]"));
        }
    }

    query_parts.join(" AND ")
}

#[cfg(test)]
mod tests {
    use rdkit::ROMol;
    use std::collections::HashMap;
    use tantivy::schema::{SchemaBuilder, FAST, STORED, TEXT};
    use tantivy::{doc, IndexBuilder};

    #[test]
    fn test_build_query() {
        let fake_descriptors: HashMap<_, _> =
            [("beep".to_string(), 1.0), ("boop".to_string(), 10.0)]
                .into_iter()
                .collect();
        let query = super::build_query(&fake_descriptors);
        assert_eq!(query, "boop: [10 TO 10000]");
    }

    #[test]
    fn test_fake_index() {
        let mut builder = SchemaBuilder::new();
        let smile_field = builder.add_text_field("smile", TEXT | STORED);
        let fingerprint_field = builder.add_bytes_field("fingerprint", FAST | STORED);
        // TODO: 3. change to be a real descriptor that rkdit would emit
        let beep_field = builder.add_i64_field("beep", FAST | STORED);

        let schema = builder.build();

        let builder = IndexBuilder::new().schema(schema.clone());
        let index = builder.create_in_ram().unwrap();

        let mut index_writer = index.writer_with_num_threads(1, 50 * 1024 * 1024).unwrap();

        let ccc_mol = ROMol::from_smile("CCC").unwrap();
        let ccc_fingerprint = ccc_mol.fingerprint();
        index_writer
            .add_document(doc!(
                smile_field => "CCC",
                fingerprint_field => ccc_fingerprint.0.into_vec(),
                beep_field => 10 as i64
            ))
            .unwrap();
        index_writer.commit().unwrap();

        let reader = index.reader().unwrap();
        let searcher = reader.searcher();

        // TODO: 1. change signature to return a list of some kind?
        super::substructure_search(&searcher, "CCCC").unwrap();

        // TODO 2. use `assert_eq` to set expectations
    }
}
