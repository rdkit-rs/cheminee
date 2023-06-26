pub use super::prelude::*;

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

    let reader = index.reader()?;
    let searcher = reader.searcher();

    let _result = basic_search(&searcher, query);

    Ok(())
}
