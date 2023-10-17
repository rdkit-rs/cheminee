use crate::indexing::open_index;
use crate::search::compound_processing::process_cpd;
use rdkit::{detect_chemistry_problems, Fingerprint, ROMol, SmilesParserParams};
use std::collections::HashMap;
use tantivy::Searcher;

pub mod basic_search;
pub mod compound_processing;
pub mod structure_matching;
pub mod substructure_search;
pub mod superstructure_search;

pub fn prepare_search(
    index_path: &String,
    smiles: &String,
) -> eyre::Result<(Searcher, ROMol, Fingerprint, HashMap<String, f64>)> {
    // Validate structure
    let problems = validate_structure(smiles);
    if !problems.is_empty() {
        panic!("Need to do something here to either correct the smiles query or return an error message");
    }

    // Need to process cpd ahead of time as we may need to iterate through tautomers later
    let (query_canon_taut, fingerprint, descriptors) = process_cpd(smiles).unwrap();

    let index = open_index(index_path)?;
    let reader = index.reader()?;
    let searcher = reader.searcher();

    Ok((searcher, query_canon_taut, fingerprint, descriptors))
}

pub fn validate_structure(smiles: &str) -> Vec<String> {
    let mut parser_params = SmilesParserParams::default();
    parser_params.sanitize(false);
    let mol = ROMol::from_smile_with_params(smiles, &parser_params).unwrap();
    detect_chemistry_problems(&mol)
}
