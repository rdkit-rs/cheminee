pub use super::prelude::*;
use crate::indexing::index_manager::IndexManager;
use std::path::Path;

pub const NAME: &str = "delete-index";

pub fn command() -> Command {
    Command::new(NAME).arg(
        Arg::new("index_path")
            .required(true)
            .long("index_path")
            .short('i')
            .num_args(1),
    )
}

pub fn action(matches: &ArgMatches) -> eyre::Result<()> {
    let index_path = matches.get_one::<String>("index_path").unwrap();
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

    let index_manager = IndexManager::new(storage_dir, false)?;
    let _ = index_manager.delete(index_name)?;

    println!("Deleted index {}", index_path);
    Ok(())
}
