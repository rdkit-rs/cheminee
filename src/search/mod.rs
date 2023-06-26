use rdkit::{detect_chemistry_problems, ROMol, SmilesParserParams};

pub mod substructure_search;
pub mod superstructure_search;

pub fn validate_structure(smiles: &String) -> Vec<String> {
    let mut parser_params = SmilesParserParams::default();
    parser_params.sanitize(false);
    let mol = ROMol::from_smile_with_params(smiles, &parser_params).unwrap();
    detect_chemistry_problems(&mol)
}