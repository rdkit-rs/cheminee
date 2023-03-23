pub mod fetch_pubchem;
pub mod index_pubchem_sdf;
pub mod rest_api_server;
pub mod search;
pub mod stream_pubchem_sdf;

pub mod prelude {
    pub use crate::tantivy::*;
    pub use clap::{Arg, ArgMatches, Command};
}
