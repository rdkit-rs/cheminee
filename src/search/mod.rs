use std::collections::{HashMap, HashSet};

use poem_openapi_derive::Object;
use rdkit::{
    detect_chemistry_problems, Fingerprint, MolSanitizeException, ROMol, SmilesParserParams,
};
use tantivy::{schema::Field, DocAddress, Searcher};

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

#[derive(Object, Debug)]
pub struct StructureSearchHit {
    pub extra_data: String,
    pub smiles: String,
    pub score: f32,
    pub query: String,
    pub used_tautomers: bool,
}

pub fn get_smiles_and_extra_data(
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

pub fn aggregate_query_hits(
    searcher: Searcher,
    results: HashSet<DocAddress>,
    query: &str,
) -> eyre::Result<Vec<QuerySearchHit>> {
    let mut final_results: Vec<QuerySearchHit> = Vec::new();
    let schema = searcher.schema();
    let smiles_field = schema.get_field("smiles")?;
    let extra_data_field = schema.get_field("extra_data")?;

    for result in results {
        let (smile, extra_data) =
            get_smiles_and_extra_data(result, &searcher, smiles_field, extra_data_field)?;

        final_results.push(QuerySearchHit {
            extra_data,
            smiles: smile.into(),
            query: query.into(),
        })
    }

    Ok(final_results)
}

pub fn aggregate_search_hits(
    searcher: Searcher,
    results: HashSet<DocAddress>,
    tautomers_used: bool,
    query: &str,
) -> eyre::Result<Vec<StructureSearchHit>> {
    let mut final_results: Vec<StructureSearchHit> = Vec::new();
    let schema = searcher.schema();
    let smiles_field = schema.get_field("smiles")?;
    let extra_data_field = schema.get_field("extra_data")?;

    let score: f32 = 1.0; // every substructure match should get a 1

    for result in results {
        let (smile, extra_data) =
            get_smiles_and_extra_data(result, &searcher, smiles_field, extra_data_field)?;

        final_results.push(StructureSearchHit {
            extra_data,
            smiles: smile,
            score,
            query: query.into(),
            used_tautomers: tautomers_used,
        })
    }

    Ok(final_results)
}
