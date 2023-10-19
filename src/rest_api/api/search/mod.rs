mod substructure_search;

use poem_openapi::payload::Json;
use poem_openapi_derive::{ApiResponse, Object};
pub use substructure_search::*;
use tantivy::{DocAddress, Searcher};

#[derive(ApiResponse)]
pub enum GetStructureSearchResponse {
    #[oai(status = "200")]
    Ok(Json<Vec<StructureSearchHit>>),
    #[oai(status = "404")]
    IndexDoesNotExist,
    #[oai(status = "500")]
    Err(Json<StructureResponseError>),
}

#[derive(Object, Debug)]
pub struct StructureResponseError {
    pub error: String,
}

#[derive(Object)]
pub struct StructureSearchHit {
    pub extra_data: serde_json::Value,
    pub smiles: String,
    pub score: f32,
    pub query: String,
    pub used_tautomers: bool,
}

#[derive(Object, Debug)]
pub struct StructureSearchResponseError {
    pub error: String,
}

pub fn aggregate_search_hits(
    searcher: Searcher,
    results: Vec<DocAddress>,
    tautomers_used: bool,
) -> eyre::Result<Vec<StructureSearchHit>> {
    let mut final_results: Vec<StructureSearchHit> = Vec::new();
    let schema = searcher.schema();
    let smile_field = schema.get_field("smile")?;
    let extra_data_field = schema.get_field("extra_data")?;

    let score: f32 = 1.0; // every substructure match should get a 1

    for result in results {
        let doc = searcher.doc(result)?;
        let smile = doc.get_first(smile_field).unwrap();
        let smile = smile.as_text().unwrap();

        let extra_data = doc.get_first(extra_data_field).unwrap();
        let extra_data = extra_data.as_text().unwrap();

        final_results.push(StructureSearchHit {
            extra_data: extra_data.into(),
            smiles: smile.to_string(),
            score: score,
            query: smile.to_string(),
            used_tautomers: tautomers_used,
        })
    }

    Ok(final_results)
}
