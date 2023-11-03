use tantivy::{collector::TopDocs, query::QueryParser, DocAddress, Searcher};

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
    let final_results = results
        .into_iter()
        .map(|result| result.1)
        .collect::<Vec<DocAddress>>();
    Ok(final_results)
}
