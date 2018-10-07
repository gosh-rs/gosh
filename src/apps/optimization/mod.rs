// mod.rs
// :PROPERTIES:
// :header-args: :tangle src/apps/optimization/mod.rs
// :END:

// [[file:~/Workspace/Programming/gosh/gosh.note::*mod.rs][mod.rs:1]]
use super::*;
use gchemol::prelude::*;
use gchemol::geometry::prelude::*;

type Point3D = [f64; 3];

pub trait Optimizer: ChemicalApp {
    /// return displacement vectors predicted by the optimizer
    fn displacements(&self, mr: &ModelProperties) -> Result<Vec<Point3D>>;

    /// test if optimization converged
    fn converged(&self, displacements: &[Point3D], mr: &ModelProperties) -> bool;

    /// Return the model for optimization
    fn model_results(&self) -> Result<ModelProperties>;

    /// Return the max allowed iterations
    fn max_iter(&self) -> usize {
        1000
    }

    fn set_displacements(&mut self, displacements: &[Point3D]);

    /// carry out optimization
    /// # Parameters
    /// - fmax: the convergence criteria of forces
    /// # Panics
    /// if fmax is not positive number.
    fn run(&mut self, fmax: f64) -> Result<()> {
        assert!(!fmax.is_sign_positive());

        let n = self.max_iter();
        let mut icycle = 0;
        loop {
            // calculate energy, forces, ... using model
            let mr = self.model_results()?;
            // calculate displacement vectors using optimizer
            let dvs = self.displacements(&mr)?;
            let forces = &mr.forces;
            if self.converged(&dvs, &mr) {
                break;
            }

            // update positions if not converged
            self.set_displacements(&dvs);

            icycle += 1;
        }
        if icycle >= n {
            warn!("opt convergence failed!");
        }

        Ok(())
    }
}

pub mod fire;
pub mod neb;
// mod.rs:1 ends here
