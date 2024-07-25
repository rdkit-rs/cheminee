use std::collections::{HashMap, HashSet};

use bitvec::prelude::{BitSlice, Lsb0};
use rdkit::{substruct_match, ROMol, SubstructMatchParameters};
use regex::Regex;
use tantivy::{DocAddress, Searcher};

use crate::search::{
    basic_search::basic_search, structure_matching::substructure_match_fp,
    STRUCTURE_MATCH_DESCRIPTORS,
};

pub fn substructure_search(
    searcher: &Searcher,
    query_mol: &ROMol,
    scaffold_matches: &Option<Vec<u64>>,
    query_fingerprint: &BitSlice<u8, Lsb0>,
    query_descriptors: &HashMap<String, f64>,
    result_limit: usize,
    extra_query: &str,
) -> eyre::Result<HashSet<DocAddress>> {
    let schema = searcher.schema();
    let query = build_substructure_query(query_descriptors, extra_query, scaffold_matches);

    // Set arbitrarily large number to force an entire database search
    let tantivy_limit = 1_000_000_000;
    let filtered_results1 = basic_search(searcher, &query, tantivy_limit)?;

    let smiles_field = schema.get_field("smiles")?;
    let fingerprint_field = schema.get_field("fingerprint")?;

    let mut filtered_results2: HashSet<DocAddress> = HashSet::new();

    for docaddr in filtered_results1 {
        if filtered_results2.len() >= result_limit {
            break;
        }

        let doc = searcher.doc(docaddr)?;

        let smiles = doc
            .get_first(smiles_field)
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
                substruct_match(&ROMol::from_smiles(smiles)?, query_mol, &params);
            if !mol_substruct_match.is_empty() && query_mol.as_smiles() != smiles {
                filtered_results2.insert(docaddr);
            }
        }
    }

    Ok(filtered_results2)
}

fn build_substructure_query(
    descriptors: &HashMap<String, f64>,
    extra_query: &str,
    matching_scaffolds: &Option<Vec<u64>>,
) -> String {
    let mut query_parts = Vec::with_capacity(descriptors.len());

    if !extra_query.is_empty() {
        for subquery in extra_query.split(" AND ") {
            query_parts.push(subquery.to_string());
        }
    }

    for (k, v) in descriptors {
        if STRUCTURE_MATCH_DESCRIPTORS.contains(&k.as_str()) {
            let re = Regex::new(&format!("{k}:")).unwrap();
            if !re.is_match(extra_query) {
                query_parts.push(format!("{k}:[{v} TO 10000]"));
            }
        }
    }

    if let Some(scaffolds) = matching_scaffolds {
        if !scaffolds.is_empty() {
            for s in scaffolds {
                query_parts.push(format!("extra_data.scaffolds:{s}"))
            }
        }
    }

    query_parts.join(" AND ")
}

#[cfg(test)]
mod tests {
    use crate::search::compound_processing::process_cpd;
    use crate::search::scaffold_search::{scaffold_search, PARSED_SCAFFOLDS};
    use crate::search::substructure_search::substructure_search;
    use serde_json::json;
    use std::collections::HashMap;
    use tantivy::schema::{JsonObjectOptions, TEXT};
    use tantivy::{
        doc,
        schema::{SchemaBuilder, FAST, INDEXED, STORED, STRING},
        IndexBuilder,
    };

    #[test]
    fn test_build_substructure_query() {
        let descriptors: HashMap<_, _> = [("NumAtoms".to_string(), 10.0)].into_iter().collect();
        let query = super::build_substructure_query(&descriptors, &"".to_string(), &None);
        assert_eq!(query, "NumAtoms:[10 TO 10000]");
    }

    #[test]
    fn test_substructure_search() {
        let index_smiles = "C1=CC=CC=C1CC2=CC=CC=C2";
        let (index_mol, index_fingerprint, index_descriptors) =
            process_cpd(index_smiles, false).unwrap();
        let index_scaffolds = scaffold_search(&index_mol, &PARSED_SCAFFOLDS).unwrap();

        let test_smiles = "C1=CC=CC=C1";
        let (query_mol, query_fingerprint, query_descriptors) =
            process_cpd(test_smiles, false).unwrap();
        let query_scaffolds = scaffold_search(&query_mol, &PARSED_SCAFFOLDS).unwrap();

        let mut builder = SchemaBuilder::new();
        let smiles_field = builder.add_text_field("smiles", STRING | STORED);
        let fingerprint_field = builder.add_bytes_field("fingerprint", FAST | STORED);

        let json_options: JsonObjectOptions =
            JsonObjectOptions::from(TEXT | STORED).set_expand_dots_enabled();
        let extra_data_field = builder.add_json_field("extra_data", json_options);

        let mut doc = doc!(
            smiles_field => index_mol.as_smiles(),
            fingerprint_field => index_fingerprint.0.clone().into_vec(),
            extra_data_field => json![{ "scaffolds": index_scaffolds }],
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

        let results = substructure_search(
            &searcher,
            &query_mol,
            &Some(query_scaffolds),
            query_fingerprint.0.as_bitslice(),
            &query_descriptors,
            10,
            &extra_query,
        )
        .unwrap();

        assert_eq!(results.len(), 1);
    }
}
