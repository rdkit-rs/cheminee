use std::path::Path;

pub mod basic_search;
pub mod bulk_index;
pub mod create_index;
pub mod delete_index;
pub mod fetch_pubchem;
pub mod index_sdf;
pub mod stream_pubchem_sdf;
pub mod substructure_search;

pub mod prelude {
    pub use clap::{Arg, ArgMatches, Command};

    pub use crate::indexing::*;
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
