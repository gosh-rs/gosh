// [[file:~/Workspace/Programming/gosh/gosh.note::*mod.rs][mod.rs:1]]
use quicli::prelude::*;

use models::{
    ChemicalModel,
    ModelProperties,
};

use gchemol::Molecule;

// sub modules
pub mod optimization;

// Application based on model chemistry
pub trait Application {
    /// Set model chemistry level
    fn set_model<T: ChemicalModel>(&mut self, model: T);

    /// Set model system
    fn set_system(&mut self, mol: &Molecule);
}
// mod.rs:1 ends here
