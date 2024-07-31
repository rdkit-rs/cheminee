use std::collections::HashMap;

use bitvec::prelude::{BitSlice, Lsb0};
use rdkit::ROMol;
use regex::Regex;
use tantivy::{DocAddress, Searcher};

use crate::search::structure_matching::exact_match;
use crate::search::{basic_search::basic_search, STRUCTURE_MATCH_DESCRIPTORS};

pub fn identity_search(
    searcher: &Searcher,
    query_mol: &ROMol,
    scaffold_matches: &Option<Vec<u64>>,
    query_fingerprint: &BitSlice<u8, Lsb0>,
    query_descriptors: &HashMap<String, f64>,
    extra_query: &str,
) -> eyre::Result<Option<DocAddress>> {
    let schema = searcher.schema();

    let query = build_identity_query(query_descriptors, extra_query, scaffold_matches);

    let tantivy_limit = 100_000;
    let initial_results = basic_search(searcher, &query, tantivy_limit)?;

    let smiles_field = schema.get_field("smiles")?;
    let fingerprint_field = schema.get_field("fingerprint")?;

    for docaddr in initial_results {
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

        let fp_match = query_fingerprint == fingerprint_bits;

        if fp_match {
            let mol_exact_match = exact_match(&ROMol::from_smiles(smiles)?, query_mol);
            if mol_exact_match {
                return Ok(Some(docaddr));
            }
        }
    }

    Ok(None)
}

pub fn build_identity_query(
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

    if let Some(scaffolds) = matching_scaffolds {
        if scaffolds.is_empty() {
            query_parts.push("extra_data.scaffolds:-1".to_string());
        } else {
            for s in scaffolds {
                query_parts.push(format!("extra_data.scaffolds:{s}"))
            }
        }
    }

    for (k, v) in descriptors {
        if STRUCTURE_MATCH_DESCRIPTORS.contains(&k.as_str()) {
            let re = Regex::new(&format!("{k}:")).unwrap();
            if !re.is_match(extra_query) {
                query_parts.push(format!("{k}:[{v} TO {v}]"));
            }
        }
    }

    query_parts.join(" AND ")
}
