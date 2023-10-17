use crate::search::compound_processing::{get_cpd_properties, get_tautomers};
use crate::search::prepare_search;
use crate::search::substructure_search::substructure_search;
use poem_openapi::payload::Json;
use poem_openapi_derive::{ApiResponse, Object};

#[derive(ApiResponse)]
pub enum GetSubstructureSearchResponse {
    #[oai(status = "200")]
    Ok(Json<Vec<SubstructureSearchHit>>),
}

#[derive(Object)]
pub struct SubstructureSearchHit {
    pub extra_data: serde_json::Value,
    pub smiles: String,
    pub score: f32,
    pub query: String,
}

// pub fn v1_index_search_substructure(
//     index: String,
//     q: Option<String>,
// ) -> GetSubstructureSearchResponse {
//     let q_str = format!("{:?}", q);
//     let index = index.to_string();
//
//     GetSubstructureSearchResponse::Ok(Json(vec![SubstructureSearchHit {
//         extra_data: serde_json::json!({"hi": "mom", "index": index}),
//         smiles: ":)".to_string(),
//         score: 100.00,
//         query: q_str,
//     }]))
// }

pub fn v1_index_search_substructure(
    index: String,
    smile: String,
    limit: usize,
) -> GetSubstructureSearchResponse {
    let index = index.to_string();
    let (searcher, query_canon_taut, fingerprint, descriptors) =
        prepare_search(&index, &smile).unwrap();

    let tantivy_result_limit = limit * 10;

    let mut results = substructure_search(
        &searcher,
        &query_canon_taut,
        fingerprint.0.as_bitslice(),
        &descriptors,
        tantivy_result_limit,
    )
    .unwrap();

    if results.len() < limit {
        let tautomers = get_tautomers(&query_canon_taut);

        let max_tauts = 10;

        for test_taut in tautomers.into_iter().take(max_tauts) {
            let (taut_fingerprint, taut_descriptors) = get_cpd_properties(&test_taut).unwrap();
            let mut taut_results = substructure_search(
                &searcher,
                &test_taut,
                taut_fingerprint.0.as_bitslice(),
                &taut_descriptors,
                tantivy_result_limit,
            )
            .unwrap();
            results.append(&mut taut_results);

            if results.len() > limit {
                break;
            }
        }
    }

    let mut final_results: Vec<SubstructureSearchHit> = Vec::new();
    let schema = searcher.schema();
    let smile_field = schema.get_field("smile").unwrap();
    let extra_data_field = schema.get_field("extra_data").unwrap();
    let score: f32 = 1.0; // every substructure match should get a 1

    for result in results {
        let doc = searcher.doc(result).unwrap();
        let smile = doc.get_first(smile_field).unwrap().as_text().unwrap();
        let extra_data = doc.get_first(extra_data_field).unwrap().as_text().unwrap();

        final_results.push(SubstructureSearchHit {
            extra_data: extra_data.into(),
            smiles: smile.to_string(),
            score: score,
            query: smile.to_string(),
        })
    }

    GetSubstructureSearchResponse::Ok(Json(final_results))
}
