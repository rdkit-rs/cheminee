use crate::search::compound_processing::process_cpd;
use poem_openapi_derive::Object;
use rdkit::{detect_chemistry_problems, Fingerprint, ROMol, SmilesParserParams};
use std::collections::HashMap;

pub mod basic_search;
pub mod compound_processing;
pub mod structure_matching;
pub mod substructure_search;
pub mod superstructure_search;

#[derive(Object, Debug)]
pub struct StructureValidationError {
    pub error: String,
}

pub fn prepare_query_structure(
    smiles: &String,
) -> eyre::Result<(ROMol, Fingerprint, HashMap<String, f64>)> {
    let problems = validate_structure(smiles);
    if !problems.is_empty() {
        panic!("Need to implement error handling here")
    };

    let (query_canon_taut, fingerprint, descriptors) = process_cpd(smiles)?;

    Ok((query_canon_taut, fingerprint, descriptors))
}

pub fn validate_structure(smiles: &str) -> Vec<String> {
    let mut parser_params = SmilesParserParams::default();
    parser_params.sanitize(false);
    let mol = ROMol::from_smile_with_params(smiles, &parser_params).unwrap();
    detect_chemistry_problems(&mol)
}
