use rdkit::ROMol;
use std::collections::HashMap;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::Searcher;

const DESCRIPTOR_ALLOW_LIST: [&'static str; 20] = [
    "NumAliphaticHeterocycles",
    "NumAliphaticRings",
    "NumAmideBonds",
    "NumAromaticHeterocycles",
    "NumAromaticRings",
    "NumAtomStereoCenters",
    "NumAtoms",
    "NumBridgeheadAtoms",
    "NumHBA",
    "NumHeavyAtoms",
    "NumHeteroatoms",
    "NumHeterocycles",
    "NumRings",
    "NumRotatableBonds",
    "NumSaturatedHeterocycles",
    "NumSaturatedRings",
    "NumSpiroAtoms",
    "NumUnspecifiedAtomStereoCenters",
    "exactmw",
    "lipinskiHBA"
];

pub fn substructure_search(searcher: &Searcher, smile: &str, limit: usize) -> eyre::Result<()> {
    let schema = searcher.schema();
    let index = searcher.index();

    let (_canonical_tautomer, _fingerprint, descriptors) =
        crate::analysis::compound_processing::process_cpd(smile)?;

    let query = build_query(&descriptors);

    let query_parser = QueryParser::for_index(index, vec![]);
    let parsed_query = query_parser.parse_query(&query)?;

    let top_docs = searcher.search(&parsed_query, &TopDocs::with_limit(limit))?;

    // (DocId, Smile, Fingerprint, DescriptorsHashMap)
    let smile_field = schema.get_field("smile")?;
    let fingerprint_field = schema.get_field("fingerprint")?;
    for (score, docaddr) in top_docs {
        let doc = searcher.doc(docaddr)?;
        let smile = doc.get_first(smile_field);
        let fingerprint = doc.get_first(fingerprint_field);

        // println!("{:?}/{:?}", smile, fingerprint);
        println!("{:?}", smile);
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
    use crate::tantivy::KNOWN_DESCRIPTORS;

    #[test]
    fn test_build_query() {
        let descriptors: HashMap<_, _> =
            [("exactmw".to_string(), 136.2), ("NumAtoms".to_string(), 10.0)]
                .into_iter()
                .collect();
        let query = super::build_query(&descriptors);
        assert_eq!(query, "exactmw: [136.2 TO 10000] AND NumAtoms: [10 TO 10000]");
    }

    #[test]
    fn test_fake_index() {
        let ccc_mol = ROMol::from_smile("CCC").unwrap();
        let ccc_fingerprint = ccc_mol.fingerprint();

        let mut builder = SchemaBuilder::new();

        let smile_field = builder.add_text_field("smile", TEXT | STORED);
        let fingerprint_field = builder.add_bytes_field("fingerprint", FAST | STORED);

        let mut doc = doc!(
            smile_field => "CCC",
            fingerprint_field => ccc_fingerprint.0.into_vec()
        );

        for field in KNOWN_DESCRIPTORS {
            if field.starts_with("Num") || field.starts_with("lipinski") {
                let current_field = builder.add_i64_field(field, FAST | STORED);

                let int = 10 as i64;
                doc.add_field_value(current_field, int);
            } else {
                let current_field = builder.add_f64_field(field, FAST | STORED);

                doc.add_field_value(
                    current_field,
                    100.0 as f64
                );
            }
        }

        let schema = builder.build();

        let builder = IndexBuilder::new().schema(schema.clone());
        let index = builder.create_in_ram().unwrap();

        let mut index_writer = index.writer_with_num_threads(1, 50 * 1024 * 1024).unwrap();

        index_writer
            .add_document(doc)
            .unwrap();
        index_writer.commit().unwrap();

        let reader = index.reader().unwrap();
        let searcher = reader.searcher();

        // TODO: 1. change signature to return a list of some kind?
        super::substructure_search(&searcher, "CCCC").unwrap();

        // TODO 2. use `assert_eq` to set expectations
    }
}
