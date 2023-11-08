pub use super::prelude::*;
use crate::indexing::index_manager::IndexManager;
use std::path::Path;

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

    let index_path_path = Path::new(index_path);
    let storage_dir = index_path_path
        .parent()
        .ok_or(eyre::eyre!("Could not extract storage directory"))?
        .to_str()
        .ok_or(eyre::eyre!("Could not convert storage director to str"))?;
    let index_name = index_path_path
        .file_stem()
        .ok_or(eyre::eyre!("Could not extract index name"))?
        .to_str()
        .ok_or(eyre::eyre!("Could not convert index name to str"))?;

    let index_manager = IndexManager::new(storage_dir, true)?;
    let _index = index_manager.create(index_name, schema, false, sort_by.map(|s| s.as_str()))?;

    println!("New index created at {}", index_path);
    Ok(())
}