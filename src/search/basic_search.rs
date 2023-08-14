use tantivy::{collector::TopDocs, query::QueryParser, DocAddress, Score, Searcher};

#[allow(clippy::ptr_arg)]
pub fn basic_search(
    searcher: &Searcher,
    query: &String,
    limit: usize,
) -> eyre::Result<Vec<(Score, DocAddress)>> {
    let index = searcher.index();
    let query_parser = QueryParser::for_index(index, vec![]);
    let query = query_parser.parse_query(query)?;
    let results = searcher.search(&query, &TopDocs::with_limit(limit))?;
    Ok(results)
}
