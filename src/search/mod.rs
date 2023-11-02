use std::collections::{HashMap, HashSet};

use poem_openapi_derive::Object;
use rdkit::{
    detect_chemistry_problems, Fingerprint, MolSanitizeException, ROMol, SmilesParserParams,
};
use tantivy::{DocAddress, Searcher};

use crate::search::compound_processing::process_cpd;

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
    smiles: &str,
) -> eyre::Result<(ROMol, Fingerprint, HashMap<String, f64>)> {
    let problems = validate_structure(smiles)?;
    if !problems.is_empty() {
        return Err(eyre::eyre!("Failed structure validation"));
    };

    let (query_canon_taut, fingerprint, descriptors) = process_cpd(smiles)?;
    Ok((query_canon_taut, fingerprint, descriptors))
}

pub fn validate_structure(smiles: &str) -> eyre::Result<Vec<MolSanitizeException>> {
    let mut parser_params = SmilesParserParams::default();
    parser_params.sanitize(false);
    let mol = ROMol::from_smile_with_params(smiles, &parser_params)?;
    Ok(detect_chemistry_problems(&mol))
}

#[derive(Object, Debug)]
pub struct StructureSearchHit {
    pub extra_data: serde_json::Value,
    pub smiles: String,
    pub score: f32,
    pub query: String,
    pub used_tautomers: bool,
}

pub fn aggregate_search_hits(
    searcher: Searcher,
    results: HashSet<DocAddress>,
    tautomers_used: bool,
    query: &str,
) -> eyre::Result<Vec<StructureSearchHit>> {
    let mut final_results: Vec<StructureSearchHit> = Vec::new();
    let schema = searcher.schema();
    let smile_field = schema.get_field("smile")?;
    let extra_data_field = schema.get_field("extra_data")?;

    let score: f32 = 1.0; // every substructure match should get a 1

    for result in results {
        let doc = searcher.doc(result)?;
        let smile = doc
            .get_first(smile_field)
            .ok_or(eyre::eyre!("Tantivy smiles retrieval failed"))?
            .as_text()
            .ok_or(eyre::eyre!("Failed to stringify smiles"))?;

        let extra_data = doc.get_first(extra_data_field);

        let extra_data = match extra_data {
            Some(extra_data) => extra_data
                .as_text()
                .ok_or(eyre::eyre!("Failed to stringify extra data"))?,
            None => "",
        };

        final_results.push(StructureSearchHit {
            extra_data: extra_data.into(),
            smiles: smile.into(),
            score,
            query: query.into(),
            used_tautomers: tautomers_used,
        })
    }

    Ok(final_results)
}
