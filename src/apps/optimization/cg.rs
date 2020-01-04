// base

// [[file:~/Workspace/Programming/gosh-rs/gosh/gosh.note::*base][base:1]]
//! Implementation of the Conjugate Gradient (CG) optimization algorithm
//!
//! # References
//! - https://en.wikipedia.org/wiki/Nonlinear_conjugate_gradient_method
//! - https://github.com/siesta-project/flos/blob/master/flos/optima/cg.lua

use super::*;

/// The beta value to determine the step of the steepest descent direction
#[derive(Debug, Clone)]
pub enum BetaKind {
    /// Polak-Ribiere
    PR,
    /// Fletcher-Reeves
    FR,
    /// Hestenes-Stiefel
    HS,
    /// Dai-Yuan
    DY,
}

impl Default for BetaKind {
    fn default() -> Self {
        BetaKind::PR
    }
}

/// Method for determining when to restart a CG optimization
#[derive(Debug, Clone)]
pub enum RestartMethod {
    /// when the scalar-projection of the two previous gradients is above 0.2
    Powell,
    /// when `beta < 0` CG restarts the conjugate gradient
    Negative,
}

impl Default for RestartMethod {
    fn default() -> Self {
        RestartMethod::Powell
    }
}

/// History data during conjugate gradient optimization
#[derive(Debug, Clone)]
struct ConjugateGradientState {
    /// The forces
    forces: Vector3fVec,
    /// The conjugate direction
    conjct: Vector3fVec,
}

#[derive(Debug, Clone)]
pub struct ConjugateGradient {
    /// The state of previous step
    state: Option<ConjugateGradientState>,

    /// Method of calculating the beta constant
    beta: BetaKind,

    /// Damping factor for creating a smooth CG restart
    beta_damping: f64,

    /// The method to restart CG optimization
    restart: RestartMethod,
}

impl Default for ConjugateGradient {
    fn default() -> Self {
        ConjugateGradient {
            state        : None,
            beta         : BetaKind::default(),
            restart      : RestartMethod::default(),
            beta_damping : 0.8,
        }
    }
}

pub type CG = ConjugateGradient;
// base:1 ends here

// core

// [[file:~/Workspace/Programming/gosh-rs/gosh/gosh.note::*core][core:1]]
impl ConjugateGradient {
    /// Return the new conjugate direction
    fn get_displacement_vectors(&mut self, forces: &Vector3fVec) -> Result<Vector3fVec> {
        // take out previous state, or update it with current forces
        let state = self.state.get_or_insert(
            ConjugateGradientState {
                forces: forces.clone(),
                conjct: forces.clone()
            }
        );

        // udpate beta
        let beta = self.beta.update(&forces, &state);

        // restart
        let beta = self.beta_damping * match self.restart {
            RestartMethod::Negative => {
                beta.max(0.0)
            },
            // Here we check whether the gradient of the current iteration has
            // "lost" orthogonality to the previous iteration
            RestartMethod::Powell => {
                let n = forces.norm_squared();
                let m = forces.dot(&state.forces);
                if n / m >= 0.2 {
                    0.0
                } else {
                    beta
                }
            },
            _ => {
                unimplemented!()
            }
        };

        // Now calculate the new steepest descent direction
        let disp = forces + beta * &state.conjct;
        let disp = scale_by_max_step(&disp);

        // save state
        state.forces = forces.clone();
        state.conjct = disp.clone();

        Ok(disp)
    }

}

fn scale_by_max_step(displacements: &Vector3fVec) -> Vector3fVec {
    let maxstep = 0.002;
    let dm = displacements.as_slice().max();
    if dm > maxstep {
        displacements * maxstep / dm
    } else {
        displacements.clone()
    }
}

impl BetaKind {
    /// Return the beta value to determine the step of the steepest descent direction
    /// # Parameters
    /// - forces: current forces
    /// - state : stored data in previous step
    fn update(&self, forces: &Vector3fVec, state: &ConjugateGradientState) -> f64 {
        let forces_this = forces;
        let forces_prev = &state.forces;
        let conjct_prev = &state.conjct;

        match self {
            BetaKind::PR => {
                forces_this.dot(&(forces_this - forces_prev)) / forces_prev.norm_squared()
            },
            BetaKind::FR => {
                forces_this.norm_squared() / forces_prev.norm_squared()
            },
            BetaKind::HS => {
                let d = forces_this - forces_prev;
                - forces_this.dot(&d) / conjct_prev.dot(&d)
            },
            BetaKind::DY => {
                let d = forces_this - forces_prev;
                forces_this.norm_squared() / conjct_prev.dot(&d)
            },
            _ => {
                error!("unkown beta parameter scheme!");
                unimplemented!()
            }
        }
    }
}
// core:1 ends here

// opt trait

// [[file:~/Workspace/Programming/gosh-rs/gosh/gosh.note::*opt trait][opt trait:1]]
impl ChemicalApp for ConjugateGradient {}

impl Optimizer for ConjugateGradient {
    /// Return cartesian displacements predicted by the optimizer
    fn displacements(&mut self, mp: &ModelProperties) -> Result<Vec<Point3D>> {
        if let Some(forces) = &mp.get_forces() {
            let forces = forces.to_dmatrix();
            let d = self.get_displacement_vectors(&forces)?;
            let d = forces_mat_to_vec(&d);
            Ok(d)
        } else {
            bail!("No forces available!");
        }
    }
}

// FIXME: dirty conversion
fn forces_mat_to_vec(forces_mat: &Vector3fVec) -> Vec<Point3D> {
    let mut forces = vec![];
    let n = forces_mat.ncols();
    for i in 0..n {
        let v = forces_mat.column(i);
        let mut p = [0.0; 3];
        p[0] = v[0];
        p[1] = v[1];
        p[2] = v[2];
        forces.push(p);
    }

    forces
}
// opt trait:1 ends here
