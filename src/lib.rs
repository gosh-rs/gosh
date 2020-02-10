// mods

// [[file:~/Workspace/Programming/gosh-rs/gosh/gosh.note::*mods][mods:1]]
pub mod cli;
// pub mod apps;
// mods:1 ends here

// pub

// [[file:~/Workspace/Programming/gosh-rs/gosh/gosh.note::*pub][pub:1]]
#[cfg(test)]
#[macro_use]
extern crate approx;

pub use gosh_model as models;

// pub mod optim {
//     // pub use crate::apps::optimization::line::golden_section_search;
// }

pub(crate) mod core {
    pub use gosh_core::gchemol;
    pub use gosh_core::gut;
    pub use gosh_core::vecfx;
}
// pub:1 ends here
