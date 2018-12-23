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



// 参考文献:
// - 袁亚湘, 非线性优化计算方法, 北京: 科学出版社, 2008, 算法2.4.5 (37页).
// - https://en.wikipedia.org/wiki/Golden-section_search


// [[file:~/Workspace/Programming/gosh/gosh.note::*base][base:2]]
/// Golden section search algorithm for unimodal function.
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
    pub fn new(a: f64, b: f64, epsilon: f64) -> Self {
        assert!(a < b, "bad section range!");
        assert!(epsilon.is_sign_positive(), "epsilon should be positive!");

        GoldenSectionSearch {
            a,
            b,
            epsilon,

            // golden ratio
            tau: 0.5 * (5f64.sqrt() + 1.0),
            ..Default::default()
        }
    }

    /// Given a function f with a single local minimum in the interval [a, b], returns a
    /// subset interval [c, d] that contains the minimum with d - c <= epsilon * 0.618.
    pub fn find<E>(&mut self, mut f: E) -> Result<(f64, f64)>
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
                }
                if self.fbeta >= self.falpha {
                    self.a = self.beta;
                }

                return Ok((self.a, self.b));
            }

            if self.fbeta > self.falpha {
                // step 3
                self.a = self.beta;
                self.beta = self.alpha;
                self.alpha = self.a + (self.b - self.a) / self.tau;
                // see python codes in Wikipedia
                self.fbeta = self.falpha;
                self.falpha = f(self.alpha)?;
            } else {
                self.b = self.alpha;
                self.alpha = self.beta;
                self.beta = self.b - (self.b - self.a) / self.tau;
                // see python codes in Wikipedia
                self.falpha = self.fbeta;
                self.fbeta = f(self.beta)?;
            }

            if self.max_iterations > 0 && k >= self.max_iterations {
                bail!("Reached max iterations!");
            }
        }

        error!("max allowed iterations!");
        Ok((self.a, self.b))
    }
}
// base:2 ends here

// test

// [[file:~/Workspace/Programming/gosh/gosh.note::*test][test:1]]
#[test]
fn test_golden_section_search() {
    let mut gss = GoldenSectionSearch::new(0.0, 2.0, 1e-6);
    let (a, b) = gss
        .find(|x| Ok(x.powi(4) - 14.0 * x.powi(3) + 60.0 * x.powi(2) - 70.0 * x))
        .expect("gss");

    relative_eq!(a, 0.7809, epsilon = 1e-4);
    relative_eq!(b, 0.7809, epsilon = 1e-4);

    let mut gss = GoldenSectionSearch::new(1.0, 5.0, 1e-5);
    let (a, b) = gss.find(|x| Ok((x - 2.0).powi(2))).expect("gss");
    relative_eq!(a, 2.0, epsilon = 1e-4);
    relative_eq!(b, 2.0, epsilon = 1e-4);
}
// test:1 ends here
