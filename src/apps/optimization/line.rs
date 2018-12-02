// base

// [[file:~/Workspace/Programming/gosh/gosh.note::*base][base:1]]
//! Line searching decides how far to step along a descent direction.
//!
//! # References:
//!
//! - Jorge Nocedal and Stephen J. Wright (2006). Numerical Optimization Springer. ISBN 0-387-30303-0.

use gchemol::geometry::prelude::*;

pub struct Backtracking {
    epsilon: f64,
}

impl Default for Backtracking {
    fn default() -> Self {
        Backtracking { epsilon: 1e-3 }
    }
}

pub struct StrongWolfe {
    /// Constant for the sufficient decrease condition.
    c1: f64,
    /// Constant for curvature condition.
    c2: f64,
}

impl Default for StrongWolfe {
    fn default() -> Self {
        StrongWolfe { c1: 0.1, c2: 0.4 }
    }
}

impl StrongWolfe {
    /// Test inexact line search conditions are satisfied or not
    /// # Parameters
    /// - feval: closure to evaluate objective function and its first derivative
    /// - xk, dk: current positions, search direction, respectively
    pub fn satisfied(&self, xk: &[f64], dk: &[f64], alpha: f64, feval: F) -> bool
    where
        F: FnMut(&[f64], &mut [f64]) -> Result<f64>,
    {
        // sanity check
        assert!(alpha.is_sign_positive());
        assert!(0.0 < self.c1 && self.c1 < self.c2 && self.c2 < 1.0);
        let n = xk.len();
        assert_eq!(n, dk.len());

        // convert to dynamic vector/matrix (nalgebra)
        let x_this = xk.to_dvector();
        let direction = dk.to_dvector();
        let mut g_this = vec![0.0; n];

        // evaluate energy and gradients
        let fx_this = feval(&x_this, &mut g_this);

        let mut g_next = vec![0.0; n];
        let x_next = x_this + alpha * direction;
        let fx_next = feval(&x_next, &mut g_next);
        // wolfe condition 1
        if fx_this - fx_next >= -self.c1 * alpha * dk * g_this {
            // wolfe condition 2
            if (dk * g_next).abs() <= (self.c2 * dk * g_this).abs() {
                true
            }
        }

        false
    }
}
// base:1 ends here

// backtracking

// [[file:~/Workspace/Programming/gosh/gosh.note::*backtracking][backtracking:1]]
/// the backtracking line search strategy starts with a relatively large step
/// size, and repeatedly shrinks it by a factor until the Armijo–Goldstein
/// condition is fulfilled.
fn backtracking_search(direction: &[f64], forces_this: &[f64], forces_prev: &[f64]) {
    unimplemented!()
}
// backtracking:1 ends here
