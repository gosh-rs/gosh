// [[file:~/Workspace/Programming/gosh/gosh.note::0f081329-2da0-4cd6-b622-265614169d68][0f081329-2da0-4cd6-b622-265614169d68]]
use super::*;

type Point3D = [f64; 3];

pub trait Optimizer {
    /// test if optimization converged
    fn converged(&self, displacements: &Vec<Point3D>, mr: &ModelResults) -> bool;

    /// Return the model for optimization
    fn model_results(&self) -> Result<ModelResults>;

    /// Return the max allowed iterations
    fn max_iter(&self) -> usize {
        1000
    }

    fn set_displacements(&mut self, displacements: &Vec<Point3D>);

    /// carry out optimization
    fn run(&mut self) -> Result<()> {
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

    /// return displacement vectors predicted by the optimizer
    fn displacements(&self, mr: &ModelResults) -> Result<Vec<Point3D>>;
}

pub mod fire;
// 0f081329-2da0-4cd6-b622-265614169d68 ends here
