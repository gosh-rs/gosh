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
    fn displacements(&self, p: &ModelProperties) -> Result<Vec<Point3D>>;

    /// Test if optimization converged
    fn converged(&self, displacements: &[Point3D], mp: &ModelProperties) -> bool;

    /// Optimize Molecule `mol` using a chemical model `model`
    /// # Parameters
    /// - fmax: the convergence criteria of forces
    /// # Panics
    /// if fmax is not positive number.
    fn run<T: ChemicalModel>(&mut self, mol: &mut Molecule, model: T) -> Result<()> {
        let mut icycle = 0;
        loop {
            info!("Optimization cycle {}", icycle);
            // calculate energy, forces, ... by applying a chemical model
            let mp = model.compute(&mol)?;

            // calculate displacement vectors using optimizer
            let dvs = self.displacements(&mp)?;
            if self.converged(&dvs, &mp) {
                info!("Optimization converged.");
                break;
            }

            // update positions if not converged
            let mut positions = mol.positions();
            let natoms = mol.natoms();
            for i in 0..natoms {
                for k in 0..3 {
                    positions[i][k] += dvs[i][k];
                }
            }
            mol.set_positions(&positions)?;

            icycle += 1;
        }

        Ok(())
    }
}

pub mod fire;
pub mod neb;
// mod.rs:1 ends here
