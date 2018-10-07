// mod.rs
// :PROPERTIES:
// :header-args: :tangle src/apps/optimization/mod.rs
// :END:

// [[file:~/Workspace/Programming/gosh/gosh.note::*mod.rs][mod.rs:1]]
use super::*;
use gchemol::prelude::*;
use gchemol::geometry::prelude::*;

type Point3D = [f64; 3];

#[derive(Debug, Clone)]
pub struct ConvergenceCriteria {
    /// The maximum number of optimization cycles.
    max_cycle: usize,
    max_force: f64,
    rms_force: f64,
}

impl Default for ConvergenceCriteria {
    fn default() -> Self {
        ConvergenceCriteria {
            max_cycle: 100,
            max_force: 0.2,
            rms_force: 0.1,
        }
    }
}

pub trait Optimizer: ChemicalApp {
    /// Return cartesian displacements predicted by the optimizer
    fn displacements(&mut self, p: &ModelProperties) -> Result<Vec<Point3D>>;

    /// Update positions with displacements
    fn set_displacements(&mut self, dvs: &[Point3D]) -> Result<()>;

    /// Define how to compute molecular properties
    fn compute_model_properties<T: ChemicalModel>(&self, model: &T) -> Result<ModelProperties>;

    /// Determine whether we have optimized the structure
    fn converged(&self, displacements: &[Point3D], mp: &ModelProperties, icycle: usize) -> Result<bool> {
        if let Some(forces) = &mp.forces {
            debug_assert!(forces.len() == displacements.len(), "vectors in different size");
            let fnorms = forces.norms();
            let dnorms = displacements.norms();

            // FIXME: criteria parameters
            let fmax = 0.03;
            let dmax = 0.05;
            let fcur = fnorms.max();
            let dcur = dnorms.max();
            if let Some(e) = &mp.energy {
                println!("{:4}\tCur Energy: {:-12.5}; Max force: {:-12.5}; Max Disp: {:-12.5}", icycle, e, fcur, dcur);
            } else {
                println!("{:4}\tMax force: {:-12.5}; Max Disp: {:-12.5}", icycle, fcur, dcur);
            }
            if fcur < fmax && dcur < dmax {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            bail!("No forces available!");
        }
    }

    /// Optimize Molecule `mol` using a chemical model `model`
    /// # Parameters
    /// - mol: The target molecule
    /// - model: The chemical model for computing molecular properties
    /// - maxcycle: The max allowed iterations
    /// # Panics
    /// if fmax is not positive number.
    fn run<T: ChemicalModel>(&mut self, model: &T, maxcycle: usize) -> Result<()> {
        let mut icycle = 0;
        loop {
            info!("Optimization cycle {}", icycle);
            // calculate energy, forces, ... by applying a chemical model
            let mp = self.compute_model_properties(model)?;

            // calculate displacement vectors using optimizer
            let dvs = self.displacements(&mp)?;
            if self.converged(&dvs, &mp, icycle)? {
                info!("Optimization converged.");
                break;
            }

            // update positions if not converged
            self.set_displacements(&dvs)?;

            icycle += 1;
            if icycle >= maxcycle {
                info!("Max allowed iteractions reached.");
                break;
            }
        }

        Ok(())
    }
}

pub mod fire;
pub mod neb;
// mod.rs:1 ends here
