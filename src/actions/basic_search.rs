pub use super::prelude::*;
use std::collections::HashMap;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;

pub const NAME: &'static str = "search";

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

    let index = open_index(index_path)?;
    let schema = index.schema();

    let reader = index.reader()?;
    let searcher = reader.searcher();

    let query_parser = QueryParser::for_index(&index, vec![]);

    let _result = basic_search(&query_parser, &searcher, &schema, query);

    Ok(())
}
