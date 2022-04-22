pub use super::prelude::*;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;

pub const NAME: &'static str = "search";

pub fn command() -> Command<'static> {
    Command::new(NAME)
        .arg(
            Arg::new("index")
                .required(true)
                .long("index")
                .short('i')
                .takes_value(true),
        )
        .arg(
            Arg::new("query")
                .required(true)
                .long("query")
                .short('q')
                .takes_value(true),
        )
}

pub fn action(matches: &ArgMatches) -> eyre::Result<()> {
    let index_path = matches.value_of("index").unwrap();
    let query = matches.value_of("query").unwrap();

    let index = open_index(index_path)?;
    let schema = index.schema();

    let smile = schema.get_field("smile").unwrap();
    let description = schema.get_field("descriptors").unwrap();

    let reader = index.reader()?;

    let searcher = reader.searcher();

    let query_parser = QueryParser::for_index(&index, vec![smile, description]);

    let query = query_parser.parse_query(query)?;

    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;

    println!("{:#?}", top_docs);

    Ok(())
}
