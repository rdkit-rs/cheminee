use std::collections::{HashMap, HashSet};

use bitvec::prelude::{BitSlice, Lsb0};
use rdkit::{substruct_match, ROMol, SubstructMatchParameters};
use regex::Regex;
use tantivy::{DocAddress, Searcher};

use crate::search::{basic_search::basic_search, structure_matching::substructure_match_fp};

const DESCRIPTOR_ALLOW_LIST: [&str; 20] = [
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
    "lipinskiHBA",
];

pub fn substructure_search(
    searcher: &Searcher,
    query_mol: &ROMol,
    query_fingerprint: &BitSlice<u8, Lsb0>,
    query_descriptors: &HashMap<String, f64>,
    result_limit: usize,
    extra_query: &String,
) -> eyre::Result<HashSet<DocAddress>> {
    let schema = searcher.schema();
    let query = build_query(query_descriptors, extra_query);

    // Note: in the end, we want a limit for the FINAL number of matches to return
    let tantivy_limit = 10 * result_limit;
    let filtered_results1 = basic_search(searcher, &query, tantivy_limit)?;

    let smile_field = schema.get_field("smile")?;
    let fingerprint_field = schema.get_field("fingerprint")?;

    let mut filtered_results2: HashSet<DocAddress> = HashSet::new();

    for docaddr in filtered_results1 {
        if filtered_results2.len() >= result_limit {
            break;
        }

        let doc = searcher.doc(docaddr)?;

        let smile = doc
            .get_first(smile_field)
            .ok_or(eyre::eyre!("Tantivy smiles retrieval failed"))?
            .as_text()
            .ok_or(eyre::eyre!("Failed to stringify smiles"))?;

        // TO-DO: find a zero-copy bitvec container
        let fingerprint = doc
            .get_first(fingerprint_field)
            .ok_or(eyre::eyre!("Tantivy fingerprint retrieval failed"))?
            .as_bytes()
            .ok_or(eyre::eyre!("Failed to read fingerprint as bytes"))?;

        let fingerprint_bits = BitSlice::<u8, Lsb0>::from_slice(fingerprint);

        let fp_match = substructure_match_fp(query_fingerprint, fingerprint_bits);

        if fp_match {
            let params = SubstructMatchParameters::default();
            let mol_substruct_match =
                substruct_match(&ROMol::from_smiles(smile)?, &query_mol, &params);
            if !mol_substruct_match.is_empty() {
                filtered_results2.insert(docaddr);
            }
        }
    }

    Ok(filtered_results2)
}

fn build_query(descriptors: &HashMap<String, f64>, extra_query: &String) -> String {
    let mut query_parts = Vec::with_capacity(descriptors.len());

    if !extra_query.is_empty() {
        for subquery in extra_query.split(" AND ") {
            query_parts.push(subquery.to_string());
        }
    }

    for (k, v) in descriptors {
        if DESCRIPTOR_ALLOW_LIST.contains(&k.as_str()) {
            let re = Regex::new(&format!("{k}:")).unwrap();
            if !re.is_match(&extra_query) {
                query_parts.push(format!("{k}: [{v} TO 10000]"));
            }
        }
    }

    query_parts.join(" AND ")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tantivy::{
        doc,
        schema::{SchemaBuilder, FAST, STORED, TEXT},
        IndexBuilder,
    };

    use crate::{indexing::KNOWN_DESCRIPTORS, search::compound_processing::process_cpd};

    #[test]
    fn test_build_query() {
        let descriptors: HashMap<_, _> = [("NumAtoms".to_string(), 10.0)].into_iter().collect();
        let query = super::build_query(&descriptors, &"".to_string());
        assert_eq!(query, "NumAtoms: [10 TO 10000]");
    }

    #[test]
    fn test_fake_index() {
        let test_smile = "C";

        let (query_mol, query_fingerprint, query_descriptors) = process_cpd(test_smile).unwrap();

        let mut builder = SchemaBuilder::new();

        let smile_field = builder.add_text_field("smile", TEXT | STORED);
        let fingerprint_field = builder.add_bytes_field("fingerprint", FAST | STORED);

        let mut doc = doc!(
            smile_field => test_smile,
            fingerprint_field => query_fingerprint.0.clone().into_vec()
        );

        for field in KNOWN_DESCRIPTORS {
            if field.starts_with("Num") || field.starts_with("lipinski") {
                let current_field = builder.add_i64_field(field, FAST | STORED);

                let int = 10_i64;
                doc.add_field_value(current_field, int);
            } else {
                let current_field = builder.add_f64_field(field, FAST | STORED);

                doc.add_field_value(current_field, 100.0_f64);
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

        let results = super::substructure_search(
            &searcher,
            &query_mol,
            query_fingerprint.0.as_bitslice(),
            &query_descriptors,
            10,
            &extra_query,
        )
        .unwrap();
        assert_eq!(results.len(), 1);
    }
}
