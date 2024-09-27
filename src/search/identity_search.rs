use crate::search::structure_matching::exact_match;
use crate::search::{basic_search::basic_search, STRUCTURE_MATCH_DESCRIPTORS};
use bitvec::prelude::{BitSlice, Lsb0};
use rayon::prelude::*;
use rdkit::ROMol;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tantivy::schema::{Field, OwnedValue};
use tantivy::{DocAddress, Searcher};

pub fn identity_search(
    searcher: &Searcher,
    query_mol: &ROMol,
    scaffold_matches: &Option<Vec<u64>>,
    query_fingerprint: &BitSlice<u8, Lsb0>,
    query_descriptors: &HashMap<String, f64>,
    use_chirality: bool,
    extra_query: &str,
) -> eyre::Result<HashSet<(String, String)>> {
    let schema = searcher.schema();

    let query = build_identity_query(query_descriptors, extra_query, scaffold_matches);

    let tantivy_limit = 100_000;
    let initial_results = basic_search(searcher, &query, tantivy_limit)?;

    let smiles_field = schema.get_field("smiles")?;
    let fingerprint_field = schema.get_field("fingerprint")?;
    let extra_data_field = schema.get_field("extra_data")?;

    let query_mol_mutex = Arc::new(Mutex::new(query_mol.clone()));

    let filtered_results = initial_results
        .into_par_iter()
        .filter_map(|result| {
            let confirmed_match = identity_match(
                result,
                smiles_field,
                fingerprint_field,
                extra_data_field,
                &searcher,
                &query_mol_mutex.lock().unwrap(),
                query_fingerprint,
                use_chirality,
            );

            if let Ok(confirmed_match) = confirmed_match {
                confirmed_match
            } else {
                None
            }
        })
        .collect::<HashSet<(String, String)>>();

    Ok(filtered_results)
}

pub fn identity_match(
    docaddr: DocAddress,
    smiles_field: Field,
    fingerprint_field: Field,
    extra_data_field: Field,
    searcher: &Searcher,
    query_mol: &ROMol,
    query_fingerprint: &BitSlice<u8>,
    use_chirality: bool,
) -> eyre::Result<Option<(String, String)>> {
    let doc = searcher.doc::<tantivy::TantivyDocument>(docaddr)?;

    let smiles = doc
        .get_first(smiles_field)
        .ok_or(eyre::eyre!("Tantivy smiles retrieval failed"))?;

    let smiles = match smiles {
        OwnedValue::Str(s) => s,
        other => return Err(eyre::eyre!("could not fetch smile, got {:?}", other)),
    };

    let fingerprint = doc
        .get_first(fingerprint_field)
        .ok_or(eyre::eyre!("Tantivy fingerprint retrieval failed"))?;

    let fingerprint = match fingerprint {
        OwnedValue::Bytes(f) => f,
        other => return Err(eyre::eyre!("could not fetch fingerprint, got {:?}", other)),
    };

    let fingerprint_bits = BitSlice::<u8, Lsb0>::from_slice(fingerprint);
    let fp_match = query_fingerprint == fingerprint_bits;

    if fp_match {
        let mol_exact_match = exact_match(&ROMol::from_smiles(smiles)?, query_mol, use_chirality);
        if mol_exact_match {
            let extra_data = match doc.get_first(extra_data_field) {
                Some(extra_data) => serde_json::to_string(extra_data)?,
                None => "".to_string(),
            };
            return Ok(Some((smiles.to_string(), extra_data)));
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
                query_parts.push(format!("{k}:{v}"));
            }
        }
    }

    query_parts.join(" AND ")
}
