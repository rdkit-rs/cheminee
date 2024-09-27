use std::collections::{HashMap, HashSet};

use crate::search::compound_processing::process_cpd;
use poem_openapi_derive::Object;
use rayon::prelude::*;
use rdkit::{
    detect_chemistry_problems, Fingerprint, MolSanitizeException, ROMol, SmilesParserParams,
};
use tantivy::schema::Field;
use tantivy::{DocAddress, Searcher};

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
    pub extra_data: String,
    pub smiles: String,
    pub query: String,
}

#[derive(Object, Debug, Clone)]
pub struct StructureSearchHit {
    pub extra_data: String,
    pub smiles: String,
    pub score: f32,
    pub query: String,
    pub used_tautomers: bool,
}

pub fn aggregate_query_hits(
    searcher: Searcher,
    results: HashSet<DocAddress>,
    query: &str,
) -> eyre::Result<Vec<QuerySearchHit>> {
    let schema = searcher.schema();
    let smiles_field = schema.get_field("smiles")?;
    let extra_data_field = schema.get_field("extra_data")?;

    let final_results = results
        .into_par_iter()
        .filter_map(|result| {
            let smiles_and_extra_data =
                get_smiles_and_extra_data(result, &searcher, smiles_field, extra_data_field);

            match smiles_and_extra_data {
                Ok((smiles, extra_data)) => Some(QuerySearchHit {
                    extra_data,
                    smiles,
                    query: query.into(),
                }),
                Err(e) => {
                    log::error!("{:?}", e);
                    None
                }
            }
        })
        .collect::<Vec<_>>();

    Ok(final_results)
}

fn get_smiles_and_extra_data(
    docaddr: DocAddress,
    searcher: &Searcher,
    smiles_field: Field,
    extra_data_field: Field,
) -> eyre::Result<(String, String)> {
    let doc = searcher.doc(docaddr)?;
    let smiles = doc
        .get_first(smiles_field)
        .ok_or(eyre::eyre!("Tantivy smiles retrieval failed"))?
        .as_text()
        .ok_or(eyre::eyre!("Failed to stringify smiles"))?;

    let extra_data = doc.get_first(extra_data_field);

    let extra_data = match extra_data {
        Some(extra_data) => serde_json::to_string(
            extra_data
                .as_json()
                .ok_or(eyre::eyre!("Failed to jsonify extra data"))?,
        )?,
        None => "".to_string(),
    };

    Ok((smiles.to_string(), extra_data.to_string()))
}
