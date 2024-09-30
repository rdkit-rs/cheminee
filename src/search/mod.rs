use std::collections::HashMap;

use crate::search::compound_processing::process_cpd;
use poem_openapi_derive::Object;
use rayon::prelude::*;
use rdkit::{
    detect_chemistry_problems, Fingerprint, MolSanitizeException, ROMol, SmilesParserParams,
};
use tantivy::schema::Field;
use tantivy::{DocAddress, DocId, Searcher, SegmentOrdinal};

pub mod basic_search;
pub mod compound_processing;
pub mod identity_search;
pub mod scaffold_search;
pub mod structure_matching;
pub mod structure_search;

#[derive(Object, Debug)]
pub struct StructureValidationError {
    pub error: String,
}

pub const STRUCTURE_MATCH_DESCRIPTORS: [&str; 19] = [
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
    "lipinskiHBA",
];

pub fn prepare_query_structure(
    smiles: &str,
) -> eyre::Result<(ROMol, Fingerprint, HashMap<String, f64>)> {
    let problems = validate_structure(smiles)?;
    if !problems.is_empty() {
        return Err(eyre::eyre!("Failed structure validation"));
    };

    let (query_canon_taut, fingerprint, descriptors) = process_cpd(smiles, false)?;
    Ok((query_canon_taut, fingerprint, descriptors))
}

pub fn validate_structure(smiles: &str) -> eyre::Result<Vec<MolSanitizeException>> {
    let mut parser_params = SmilesParserParams::default();
    parser_params.set_sanitize(false);
    let mol =
        ROMol::from_smiles_with_params(smiles, &parser_params).map_err(|e| eyre::eyre!("{}", e))?;
    Ok(detect_chemistry_problems(&mol))
}

#[derive(Object, Debug)]
pub struct QuerySearchHit {
    pub extra_data: serde_json::Value,
    pub smiles: String,
    pub query: String,
}

#[derive(Object, Debug, Clone)]
pub struct StructureSearchHit {
    pub extra_data: serde_json::Value,
    pub smiles: String,
    pub score: f32,
    pub query: String,
    pub used_tautomers: bool,
}

pub fn aggregate_query_hits(
    searcher: Searcher,
    results: Vec<DocAddress>,
    query: &str,
) -> eyre::Result<Vec<QuerySearchHit>> {
    let schema = searcher.schema();
    let smiles_field = schema.get_field("smiles")?;
    let extra_data_field = schema.get_field("extra_data")?;

    let mut data_results = results
        .into_par_iter()
        .filter_map(|result| {
            let smiles_and_extra_data =
                get_smiles_and_extra_data(result, &searcher, smiles_field, extra_data_field);

            match smiles_and_extra_data {
                Ok((smiles, extra_data)) => {
                    Some((smiles, extra_data, result.segment_ord, result.doc_id))
                }
                Err(e) => {
                    log::error!("{:?}", e);
                    None
                }
            }
        })
        .collect::<Vec<_>>();

    let sorted_results = sort_results(&mut data_results)
        .into_iter()
        .map(|(smiles, extra_data)| QuerySearchHit {
            extra_data,
            smiles,
            query: query.into(),
        })
        .collect::<Vec<_>>();

    Ok(sorted_results)
}

fn get_smiles_and_extra_data(
    docaddr: DocAddress,
    searcher: &Searcher,
    smiles_field: Field,
    extra_data_field: Field,
) -> eyre::Result<(String, serde_json::Value)> {
    let doc = searcher.doc::<tantivy::TantivyDocument>(docaddr)?;
    let smiles = doc
        .get_first(smiles_field)
        .ok_or(eyre::eyre!("Tantivy smiles retrieval failed"))?;

    let smiles = match smiles {
        tantivy::schema::OwnedValue::Str(s) => s,
        other => return Err(eyre::eyre!("expect string got {:?}", other)),
    };

    // TODO: sure would be nice if it was easier to take a tantivy OwnedValue and turn it in to a serde_json Object!
    let extra_data = doc.get_first(extra_data_field);
    let extra_data = match extra_data {
        Some(tantivy::schema::OwnedValue::Object(obj_map)) => {
            serde_json::from_str(&serde_json::to_string(&obj_map)?)?
        }
        Some(_) | None => serde_json::Value::Object(Default::default()),
    };

    Ok((smiles.to_string(), extra_data))
}

pub fn sort_results(
    results: &mut [(String, serde_json::Value, SegmentOrdinal, DocId)],
) -> Vec<(String, serde_json::Value)> {
    results.sort_by(|a, b| {
        let cmp = a.2.cmp(&b.2);

        if cmp == std::cmp::Ordering::Equal {
            a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal)
        } else {
            cmp
        }
    });

    results
        .iter_mut()
        .map(|(data1, data2, _, _)| (data1.to_owned(), data2.to_owned()))
        .collect::<Vec<_>>()
}
