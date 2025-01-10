use crate::command_line::prelude::*;
use crate::search::basic_search::basic_search_count;

pub const NAME: &str = "basic-search-count";

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
    let index_path = matches
        .get_one::<String>("index")
        .ok_or(eyre::eyre!("Failed to extract index path"))?;
    let query = matches
        .get_one::<String>("query")
        .ok_or(eyre::eyre!("Failed to extract query"))?;

    let index = open_index(index_path)?;
    let reader = index.reader()?;
    let searcher = reader.searcher();
    let result_count = basic_search_count(&searcher, query)?;

    log::info!("{:#?}", result_count);

    Ok(())
}
