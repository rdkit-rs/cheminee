use crate::search::compound_processing::get_cpd_properties;
use crate::search::scaffold_search::{scaffold_search, PARSED_SCAFFOLDS};
use crate::search::{
    basic_search::basic_search, structure_matching::substructure_match_fp,
    STRUCTURE_MATCH_DESCRIPTORS,
};
use bitvec::prelude::{BitSlice, Lsb0};
use rayon::prelude::*;
use rdkit::{substruct_match, ROMol, SubstructMatchParameters};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tantivy::schema::Field;
use tantivy::{DocAddress, DocId, Searcher, SegmentOrdinal};

pub fn structure_search(
    searcher: &Searcher,
    query_mol: &ROMol,
    method: &str,
    use_scaffolds: bool,
    result_limit: usize,
    use_chirality: bool,
    extra_query: &str,
) -> eyre::Result<HashSet<(String, serde_json::Value, SegmentOrdinal, DocId)>> {
    let schema = searcher.schema();

    let (query_pattern_fingerprint, query_descriptors) = get_cpd_properties(query_mol)?;

    let query_pattern_fingerprint = query_pattern_fingerprint.0.as_bitslice();

    let scaffold_matches = if use_scaffolds {
        Some(scaffold_search(
            query_pattern_fingerprint,
            query_mol,
            &PARSED_SCAFFOLDS,
        )?)
    } else {
        None
    };

    let query = if method == "substructure" {
        build_substructure_query(&query_descriptors, extra_query, &scaffold_matches)
    } else {
        build_superstructure_query(&query_descriptors, extra_query, &scaffold_matches)
    };

    let tantivy_limit = 100_000;
    let initial_results = basic_search(searcher, &query, tantivy_limit)?;

    let smiles_field = schema.get_field("smiles")?;
    let pattern_fingerprint_field = schema.get_field("pattern_fingerprint")?;
    let extra_data_field = schema.get_field("extra_data")?;

    let query_mol_mutex = Arc::new(Mutex::new(query_mol.clone()));

    let mut result_count: usize = 0;
    let mut filtered_results: HashSet<(String, serde_json::Value, SegmentOrdinal, DocId)> =
        HashSet::new();

    for chunk in initial_results.chunks(1000) {
        if result_count > result_limit {
            break;
        }

        let results_subset = chunk
            .into_par_iter()
            .filter_map(|result| {
                let struct_match = structure_match(
                    *result,
                    smiles_field,
                    pattern_fingerprint_field,
                    extra_data_field,
                    searcher,
                    &query_mol_mutex.lock().unwrap(),
                    query_pattern_fingerprint,
                    method,
                    use_chirality,
                );

                struct_match.unwrap_or_else(|e| {
                    log::error!("{:?}", e);
                    None
                })
            })
            .collect::<HashSet<_>>();

        result_count += results_subset.len();
        filtered_results.extend(results_subset);
    }

    Ok(filtered_results)
}

#[allow(clippy::too_many_arguments)]
pub fn structure_match(
    docaddr: DocAddress,
    smiles_field: Field,
    pattern_fingerprint_field: Field,
    extra_data_field: Field,
    searcher: &Searcher,
    query_mol: &ROMol,
    query_pattern_fingerprint: &BitSlice<u8>,
    method: &str,
    use_chirality: bool,
) -> eyre::Result<Option<(String, serde_json::Value, SegmentOrdinal, DocId)>> {
    let doc = searcher.doc::<tantivy::TantivyDocument>(docaddr)?;

    let smiles = doc
        .get_first(smiles_field)
        .ok_or(eyre::eyre!("Tantivy smiles retrieval failed"))?;

    let smiles = match smiles {
        tantivy::schema::OwnedValue::Str(s) => s,
        other => return Err(eyre::eyre!("expected string, got {:?}", other)),
    };

    // TO-DO: find a zero-copy bitvec container
    let pattern_fingerprint = doc
        .get_first(pattern_fingerprint_field)
        .ok_or(eyre::eyre!("Tantivy pattern fingerprint retrieval failed"))?;

    let pattern_fingerprint = match pattern_fingerprint {
        tantivy::schema::OwnedValue::Bytes(b) => b,
        other => return Err(eyre::eyre!("expected bytes, got {:?}", other)),
    };

    let pattern_fingerprint_bits = BitSlice::<u8, Lsb0>::from_slice(pattern_fingerprint);

    let fp_match = if method == "substructure" {
        substructure_match_fp(query_pattern_fingerprint, pattern_fingerprint_bits)
    } else {
        substructure_match_fp(pattern_fingerprint_bits, query_pattern_fingerprint)
    };

    if fp_match {
        let mut params = SubstructMatchParameters::default();
        params.set_use_chirality(use_chirality);

        let mol_substruct_match = if method == "substructure" {
            substruct_match(&ROMol::from_smiles(smiles)?, query_mol, &params)
        } else {
            substruct_match(query_mol, &ROMol::from_smiles(smiles)?, &params)
        };

        if !mol_substruct_match.is_empty() && query_mol.as_smiles() != *smiles {
            let extra_data = match doc.get_first(extra_data_field) {
                Some(extra_data) => serde_json::from_str(&serde_json::to_string(extra_data)?)?,
                None => serde_json::Value::Object(Default::default()),
            };

            return Ok(Some((
                smiles.to_string(),
                extra_data,
                docaddr.segment_ord,
                docaddr.doc_id,
            )));
        }
    }

    Ok(None)
}

pub fn build_substructure_query(
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
                query_parts.push(format!("other_descriptors.scaffolds:{s}"))
            }
        }
    }

    query_parts.join(" AND ")
}

pub fn build_superstructure_query(
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
                query_parts.push(format!("{k}:[0 TO {v}]"));
            }
        }
    }

    let mut query = query_parts.join(" AND ");

    if let Some(scaffolds) = matching_scaffolds {
        if scaffolds.is_empty() {
            query = format!("{query} AND other_descriptors.scaffolds:-1");
        } else {
            let mut scaffold_parts = scaffolds
                .iter()
                .map(|s| format!("other_descriptors.scaffolds:{s}"))
                .collect::<Vec<String>>();

            scaffold_parts.push("other_descriptors.scaffolds:-1".to_string());

            let scaffolds_query = scaffold_parts.join(" OR ");

            query = format!("{query} AND ({scaffolds_query})");
        }
    }

    query
}
