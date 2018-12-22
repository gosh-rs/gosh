// header

// [[file:~/Workspace/Programming/gosh/gosh.note::*header][header:1]]
//! Line searching decides how far to step along a descent direction.
//!
//! # References:
//!
//! - Jorge Nocedal and Stephen J. Wright (2006). Numerical Optimization Springer. ISBN 0-387-30303-0.

use super::*;
use gchemol::geometry::prelude::*;
// header:1 ends here

// MoreThuente
// ported from lbfgs.c

// [[file:~/Workspace/Programming/gosh/gosh.note::*MoreThuente][MoreThuente:1]]

// MoreThuente:1 ends here

// backtracking

// [[file:~/Workspace/Programming/gosh/gosh.note::*backtracking][backtracking:1]]
/// the backtracking line search strategy starts with a relatively large step
/// size, and repeatedly shrinks it by a factor until the Armijo–Goldstein
/// condition is fulfilled.
fn backtracking_search(direction: &[f64], forces_this: &[f64], forces_prev: &[f64]) {
    unimplemented!()
}
// backtracking:1 ends here

// base
// 参考文献: 袁亚湘, 非线性优化计算方法, 北京: 科学出版社, 2008, 算法2.4.5 (37页).

// [[file:~/Workspace/Programming/gosh/gosh.note::*base][base:1]]
#[derive(Clone, Debug, Default)]
pub struct GoldenSectionSearch {
    /// Max allowed iterations. If set it as 0, the iteration will loop forever.
    pub max_iterations: usize,

    /// accuray tolerance
    pub epsilon: f64,

    pub a: f64,
    pub b: f64,

    alpha: f64,
    beta: f64,

    falpha: f64,
    fbeta: f64,

    tau: f64,
}

impl GoldenSectionSearch {
    pub fn new(a: f64, b: f64) -> Self {
        assert!(a >= b || !a.is_sign_positive(), "bad section range!");

        GoldenSectionSearch {
            // golden ratio
            tau: 0.5 * (5f64.sqrt() + 1.0),
            epsilon: 1e-5,
            a,
            b,

            ..Default::default()
        }
    }

    /// find a satisfactory position between point `a` and `b` using the Golden
    /// Section Search algorithm
    pub fn find<E>(&mut self, mut f: E) -> Result<()>
    where
        E: FnMut(f64) -> Result<f64>,
    {
        // step 1
        ensure!(
            self.a < self.b && self.epsilon.is_sign_positive(),
            "GSS: bad params!"
        );

        self.alpha = self.a + (self.b - self.a) / self.tau;
        self.beta = self.a + self.b - self.alpha;

        self.falpha = f(self.alpha)?;
        self.fbeta = f(self.beta)?;

        let tol = self.tau * self.epsilon;

        // step 2
        for k in 1.. {
            // convergence test
            if self.b - self.a <= tol {
                // step 4
                if self.fbeta <= self.falpha {
                    self.b = self.alpha;
                } else {
                    self.a = self.beta;
                }
                break;
            }

            if self.fbeta > self.falpha {
                // step 3
                self.a = self.beta;
                self.beta = self.alpha;
                self.alpha = self.a + (self.b - self.a) / self.tau;
                self.falpha = f(self.alpha)?;
            } else {
                self.b = self.alpha;
                self.alpha = self.beta;
                self.beta = self.b - (self.b - self.a) / self.tau;
                self.fbeta = f(self.beta)?;
            }

            if k >= self.max_iterations {
                bail!("Reached max iterations!");
            }
        }

        Ok(())
    }
}
// base:1 ends here
