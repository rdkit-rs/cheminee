pub mod indexing;
pub mod pubchem;
pub mod search;

pub mod prelude {
    pub use crate::indexing::*;
    pub use clap::{Arg, ArgMatches, Command};
}
