pub use super::prelude::*;
use crate::command_line::indexing::split_path;
use crate::indexing::index_manager::IndexManager;
use std::ops::Deref;

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
    let index_path = matches.get_one::<String>("index_path")?;
    let (storage_dir, index_name) = split_path(index_path)?;

    let index_manager = IndexManager::new(storage_dir, false)?;
    let _ = index_manager.delete(index_name.deref())?;

    println!("Deleted index {}", index_path);
    Ok(())
}
