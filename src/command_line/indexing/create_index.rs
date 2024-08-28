use crate::command_line::indexing::split_path;
use crate::command_line::prelude::*;
use crate::indexing::index_manager::IndexManager;
use std::ops::Deref;

pub const NAME: &str = "create-index";

pub fn command() -> Command {
    Command::new(NAME)
        .arg(
            Arg::new("index-path")
                .required(true)
                .long("index-path")
                .short('i')
                .num_args(1),
        )
        .arg(
            Arg::new("schema-name")
                .required(true)
                .long("schema-name")
                .short('n')
                .num_args(1),
        )
        .arg(
            Arg::new("sort-by")
                .required(false)
                .long("sort-by")
                .short('s')
                .num_args(1),
        )
}

pub fn action(matches: &ArgMatches) -> eyre::Result<()> {
    let index_path = matches
        .get_one::<String>("index-path")
        .ok_or(eyre::eyre!("Failed to extract index path"))?;
    let schema_name = matches
        .get_one::<String>("schema-name")
        .ok_or(eyre::eyre!("Failed to extract schema name"))?;
    let sort_by = matches.get_one::<String>("sort-by");

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

    log::info!("New index created at {}", index_path);
    Ok(())
}
