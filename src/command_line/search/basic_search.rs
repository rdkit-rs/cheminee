use crate::command_line::prelude::*;
use crate::search::aggregate_query_hits;
use crate::search::basic_search::basic_search;

pub const NAME: &str = "basic-search";

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
    let index_path = matches
        .get_one::<String>("index")
        .ok_or(eyre::eyre!("Failed to extract index path"))?;
    let query = matches
        .get_one::<String>("query")
        .ok_or(eyre::eyre!("Failed to extract query"))?;
    let limit = matches.get_one::<String>("limit");

    let limit = if let Some(limit) = limit {
        limit.parse::<usize>()?
    } else {
        usize::try_from(1000)?
    };

    let index = open_index(index_path)?;
    let reader = index.reader()?;
    let searcher = reader.searcher();

    let results = basic_search(&searcher, query, limit)?;

    let final_results = aggregate_query_hits(searcher, results, query)?;

    println!("{:#?}", final_results);

    Ok(())
}
