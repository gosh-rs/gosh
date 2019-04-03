// base

// [[file:~/Workspace/Programming/gosh-rs/gosh/gosh.note::*base][base:1]]
#![feature(test)]
#[macro_use]
extern crate nom;

#[cfg(test)]
#[macro_use]
extern crate approx;

pub mod adaptors;
pub mod apps;
pub use gchemol;
pub use gosh_models as models;

pub mod core_utils {
    pub use quicli::prelude::*;
    pub type Result<T> = ::std::result::Result<T, Error>;
}

pub mod cmd_utils {
    pub use crate::core_utils::*;
    pub use structopt::StructOpt;
}

pub mod optim {
    pub use crate::apps::optimization::line::golden_section_search;
}
// base:1 ends here
