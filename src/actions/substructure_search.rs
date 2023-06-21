pub use super::prelude::*;
use std::collections::HashMap;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;

pub const NAME: &'static str = "substructure-search";

pub fn command() -> Command {
    Command::new(NAME)
        .arg(
            Arg::new("index")
                .required(true)
                .long("index")
                .short('i')
                .num_args(1),
        )
        .arg(
            Arg::new("query")
                .required(true)
                .long("query")
                .short('q')
                .num_args(1),
        )
}

pub fn action(matches: &ArgMatches) -> eyre::Result<()> {
    let index_path = matches.get_one::<String>("index").unwrap();
    let query = matches.get_one::<String>("query").unwrap();

    let index = open_index(index_path)?;
    let schema = index.schema();

    let reader = index.reader()?;
    let searcher = reader.searcher();

    let query_parser = QueryParser::for_index(&index, vec![]);
    let query = query_parser.parse_query(query)?;

    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;

    let docs = top_docs
        .into_iter()
        .map(|(_score, doc_addr)| {
            let doc = searcher.doc(doc_addr).unwrap();
            let field_values = doc.field_values();
            let reconstituted_doc = field_values
                .iter()
                .map(|field_value| {
                    let field_name = schema.get_field_name(field_value.field);
                    (field_name, field_value.value.clone())
                })
                .collect::<HashMap<_, _>>();

            (doc_addr, reconstituted_doc)
        })
        .collect::<Vec<_>>();

    for doc in docs {
        println!("{:?}", doc);
    }

    Ok(())
}
