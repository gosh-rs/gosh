// mods

// [[file:~/Workspace/Programming/gosh-rs/gosh/gosh.note::*mods][mods:1]]
pub mod apps;
pub mod cli;

#[cfg(test)]
#[macro_use]
extern crate approx;

pub use gosh_models as models;

pub mod cmd_utils {
    pub use crate::core::*;

    // pub use guts::cli::*;
    pub use quicli::prelude::*;

    pub use structopt::StructOpt;
}

pub mod optim {
    pub use crate::apps::optimization::line::golden_section_search;
}

pub mod core {
    pub use gosh_core::*;

    pub use guts::prelude::*;
}
// mods:1 ends here
