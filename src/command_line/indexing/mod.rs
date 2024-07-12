use std::path::Path;

pub mod bulk_delete;
pub mod bulk_index;
pub mod create_index;
pub mod delete_index;
pub mod index_sdf;

pub mod prelude {
    pub use crate::indexing::*;
    pub use clap::{Arg, ArgMatches, Command};
}

pub fn split_path(index_path: &String) -> eyre::Result<(String, String)> {
    let index_path_path = Path::new(index_path);
    let parent_dir = index_path_path
        .parent()
        .ok_or(eyre::eyre!("Could not extract storage directory"))?
        .to_str()
        .ok_or(eyre::eyre!("Could not convert storage director to str"))?
        .to_string();
    let child_name = index_path_path
        .file_stem()
        .ok_or(eyre::eyre!("Could not extract index name"))?
        .to_str()
        .ok_or(eyre::eyre!("Could not convert index name to str"))?
        .to_string();

    Ok((parent_dir, child_name))
}
