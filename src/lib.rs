// [[file:../gosh.note::*mods][mods:1]]
mod bbm;
mod cli;
mod repl;
// mods:1 ends here

// [[file:../gosh.note::*pub][pub:1]]
#[cfg(test)]
#[macro_use]
extern crate approx;

pub use gosh_adaptor as adaptor;
pub use gosh_core::gchemol;
pub use gosh_database as db;
pub use gosh_model as model;
pub use gosh_optim as optim;
pub use gosh_runner as runner;

pub(crate) mod common {
    pub use gosh_core::gchemol;
    pub use gosh_core::gut;
    pub use gosh_core::gut::prelude::*;
    pub use gosh_core::vecfx;

    pub use std::path::{Path, PathBuf};
}

pub mod prelude {
    pub use gosh_database::prelude::*;
}

pub use bbm::bbm_enter_main;
pub use repl::repl_enter_main;
// pub:1 ends here
