pub use super::prelude::*;
use crate::command_line::split_path;
use crate::indexing::index_manager::IndexManager;
use std::ops::Deref;

pub const NAME: &str = "create-index";

pub fn command() -> Command {
    Command::new(NAME)
        .arg(
            Arg::new("index_path")
                .required(true)
                .long("index_path")
                .short('i')
                .num_args(1),
        )
        .arg(
            Arg::new("schema_name")
                .required(true)
                .long("schema_name")
                .short('n')
                .num_args(1),
        )
        .arg(
            Arg::new("sort_by")
                .required(false)
                .long("sort_by")
                .short('s')
                .num_args(1),
        )
}

pub fn action(matches: &ArgMatches) -> eyre::Result<()> {
    let index_path = matches.get_one::<String>("index_path").unwrap();
    let schema_name = matches.get_one::<String>("schema_name").unwrap();
    let sort_by = matches.get_one::<String>("sort_by");

    let schema = crate::schema::LIBRARY
        .get(schema_name.as_str())
        .ok_or(eyre::eyre!("The specified schema does not exist"))?;

    let (storage_dir, index_name) = split_path(index_path)?;
    let index_manager = IndexManager::new(storage_dir.deref(), true)?;
    let _index = index_manager.create(
        index_name.deref(),
        schema,
        false,
        sort_by.map(|s| s.as_str()),
    )?;

    println!("New index created at {}", index_path);
    Ok(())
}
