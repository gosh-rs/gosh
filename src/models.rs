// [[file:~/Workspace/Programming/gosh/gosh.note::894d0c1b-0482-46b9-a3dc-8f00b78833bc][894d0c1b-0482-46b9-a3dc-8f00b78833bc]]
use quicli::prelude::*;

pub struct ModelResults {
    energy          : Option<f64>,
    forces          : Option<Vec<[f64; 3]>>,
    dipole_moment   : Option<[f64; 3]>,
    force_constants : Option<Vec<[f64; 3]>>,
}

impl Default for ModelResults {
    fn default() -> Self {
        ModelResults {
            energy          : None,
            forces          : None,
            dipole_moment   : None,
            force_constants : None,
        }
    }
}

pub trait ChemicalModel {
    fn positions(&self) -> Vec<[f64; 3]> {
        unimplemented!()
    }

    /// define how to calculate properties, such as energy, forces, ...
    fn calculate(&self) -> Result<()> {
        unimplemented!()
    }

    fn energy(&self) -> f64 {
        unimplemented!()
    }

    fn forces(&self) -> Vec<[f64; 3]> {
        unimplemented!()
    }

    fn dipole_moment(&self) -> [f64; 3] {
        unimplemented!()
    }

    // fn polarizability(&self) ->
    // fn dipole_derivatives(&self) ->
    // fn force_constants(&self) ->
}
// 894d0c1b-0482-46b9-a3dc-8f00b78833bc ends here

// [[file:~/Workspace/Programming/gosh/gosh.note::ea1864bb-6cc4-42f1-93f5-cebd790c58ab][ea1864bb-6cc4-42f1-93f5-cebd790c58ab]]

// ea1864bb-6cc4-42f1-93f5-cebd790c58ab ends here
