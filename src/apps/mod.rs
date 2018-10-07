// mod.rs
// :PROPERTIES:
// :header-args: :tangle src/apps/mod.rs
// :END:

// [[file:~/Workspace/Programming/gosh/gosh.note::*mod.rs][mod.rs:1]]
use quicli::prelude::*;

use crate::models::{
    ChemicalModel,
    ModelProperties,
};

use gchemol::Molecule;

// sub modules
pub mod optimization;

// Application based on model chemistry
pub trait ChemicalApp {
    /// Set model chemistry level
    fn set_model<T: ChemicalModel>(&mut self, model: T);

    /// Set model system
    fn set_system(&mut self, mol: &Molecule);
}
// mod.rs:1 ends here
