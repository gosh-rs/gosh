// base

// [[file:~/Workspace/Programming/gosh/gosh.note::*base][base:1]]
#![feature(test)]
#[macro_use] extern crate nom;
#[macro_use] extern crate gchemol;
#[macro_use] extern crate duct;

#[cfg(test)]
#[macro_use] extern crate approx;

pub mod models;
pub mod adaptors;
pub mod apps;

pub mod core_utils {
    pub use quicli::prelude::*;
    pub type Result<T> = ::std::result::Result<T, Error>;
}

pub mod cmd_utils {
    pub use crate::core_utils::*;
    pub use ::structopt::StructOpt;
}
// base:1 ends here
