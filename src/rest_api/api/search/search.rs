use poem_openapi::payload::Json;
use poem_openapi_derive::{ApiResponse, Object};

use crate::{
    indexing::index_manager::IndexManager,
    rest_api::{
        api::{aggregate_search_hits, GetStructureSearchResponse, StructureResponseError},
        models::GenericResponseError,
    },
    search::{
        compound_processing::{get_cpd_properties, get_tautomers},
        prepare_query_structure,
        substructure_search::substructure_search,
    },
};

#[derive(ApiResponse)]
pub enum GetIndexesSearchResponse {
    #[oai(status = "200")]
    Ok(Json<Vec<SearchHit>>),
    #[oai(status = "404")]
    IndexDoesNotExist,
    #[oai(status = "500")]
    Err(Json<GenericResponseError>),
}

#[derive(Object, Debug)]
pub struct SearchHit {
    pub error: String,
}

pub fn v1_index_search(
    index_manager: &IndexManager,
    index: String,
    query: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> GetIndexesSearchResponse {
    if !index_manager.exists(&index) {
        return GetIndexesSearchResponse::IndexDoesNotExist;
    }

    let index = index_manager.open(&index);

    let index = match index {
        Ok(i) => i,
        Err(e) => return GetIndexesSearchResponse::Err(Json(e.into())),
    };

    let query_parser = tantivy::query::QueryParser::for_index(&index, vec![]);
    let schema = index.schema();

    let reader = match index.reader() {
        Ok(r) => r,
        Err(e) => return GetIndexesSearchResponse::Err(e.into()),
    };

    let searcher = reader.searcher();

    let query = query_parser.parse_query(query.as_deref().unwrap());
    let query = match query {
        Ok(q) => q,
        Err(e) => return GetIndexesSearchResponse::Err(e.into()),
    };

    let collector = tantivy::collector::TopDocs::with_limit(1000)
    searcher.search(&query)
}
