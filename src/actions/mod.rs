pub mod basic_search;
pub mod fetch_pubchem;
pub mod index_pubchem_sdf;
pub mod rest_api_server;
pub mod stream_pubchem_sdf;
pub mod substructure_search;

pub mod prelude {
    pub use crate::tantivy::*;
    pub use clap::{Arg, ArgMatches, Command};
}
