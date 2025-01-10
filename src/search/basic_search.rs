use crate::search::sort_docs;
use rayon::prelude::*;
use tantivy::collector::{Count, TopDocs};
use tantivy::{query::QueryParser, DocAddress, Searcher};

#[allow(clippy::ptr_arg)]
pub fn basic_search(
    searcher: &Searcher,
    query: &String,
    limit: usize,
) -> eyre::Result<Vec<DocAddress>> {
    let index = searcher.index();
    let query_parser = QueryParser::for_index(index, vec![]);
    let query = query_parser.parse_query(query)?;
    let results = searcher.search(&query, &TopDocs::with_limit(limit))?;
    let mut final_results = results
        .into_par_iter()
        .map(|result| result.1)
        .collect::<Vec<_>>();

    sort_docs(&mut final_results);
    Ok(final_results)
}

#[allow(clippy::ptr_arg)]
pub fn basic_search_count(searcher: &Searcher, query: &String) -> eyre::Result<usize> {
    let index = searcher.index();
    let query_parser = QueryParser::for_index(index, vec![]);
    let query = query_parser.parse_query(query)?;
    let result_count = searcher.search(&query, &Count)?;

    Ok(result_count)
}
