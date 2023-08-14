pub use super::prelude::*;
use crate::search::basic_search::basic_search;

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
        .arg(
            Arg::new("limit")
                .required(false)
                .long("limit")
                .short('l')
                .num_args(1),
        )
}

pub fn action(matches: &ArgMatches) -> eyre::Result<()> {
    let index_path = matches.get_one::<String>("index").unwrap();
    let query = matches.get_one::<String>("query").unwrap();
    let limit = matches.get_one::<String>("limit");

    let index = open_index(index_path)?;
    let reader = index.reader()?;
    let searcher = reader.searcher();

    let limit = if let Some(limit) = limit {
        limit.parse::<usize>()?
    } else {
        usize::try_from(1000).unwrap()
    };

    let results = basic_search(&searcher, query, limit);

    println!("{:#?}", results);

    Ok(())
}
