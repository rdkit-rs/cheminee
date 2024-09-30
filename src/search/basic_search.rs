use rayon::prelude::*;
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
    let mut final_results = results
        .into_par_iter()
        .map(|result| result.1)
        .collect::<Vec<_>>();

    sort_docs(&mut final_results);
    Ok(final_results)
}

fn sort_docs(results: &mut [DocAddress]) {
    results.sort_by(|a, b| {
        let cmp = a.segment_ord.cmp(&b.segment_ord);

        if cmp == std::cmp::Ordering::Equal {
            a.doc_id
                .partial_cmp(&b.doc_id)
                .unwrap_or(std::cmp::Ordering::Equal)
        } else {
            cmp
        }
    });
}
