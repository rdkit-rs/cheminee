use crate::indexing::index_manager::IndexManager;
use crate::rest_api::api::{GetQuerySearchResponse, QueryResponseError};
use crate::search::aggregate_query_hits;
use crate::search::basic_search::basic_search;
use poem_openapi::payload::Json;

pub fn v1_index_search_basic(
    index_manager: &IndexManager,
    index: String,
    query: String,
    limit: usize,
) -> GetQuerySearchResponse {
    let index = match index_manager.open(&index) {
        Ok(index) => index,
        Err(e) => {
            return GetQuerySearchResponse::Err(Json(QueryResponseError {
                error: e.to_string(),
            }))
        }
    };

    let reader = index.reader();
    let reader = match reader {
        Ok(reader) => reader,
        Err(e) => {
            return GetQuerySearchResponse::Err(Json(QueryResponseError {
                error: e.to_string(),
            }))
        }
    };

    let searcher = reader.searcher();

    let tantivy_limit = 10 * limit;
    let results = basic_search(&searcher, &query, tantivy_limit);

    let results = match results {
        Ok(results) => results,
        Err(e) => {
            return GetQuerySearchResponse::Err(Json(QueryResponseError {
                error: e.to_string(),
            }))
        }
    };

    let final_results = aggregate_query_hits(searcher, results, &query);

    let final_results = match final_results {
        Ok(final_results) => final_results,
        Err(e) => {
            return GetQuerySearchResponse::Err(Json(QueryResponseError {
                error: e.to_string(),
            }))
        }
    };

    GetQuerySearchResponse::Ok(Json(final_results))
}
