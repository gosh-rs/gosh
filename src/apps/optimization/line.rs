// base

// [[file:~/Workspace/Programming/gosh/gosh.note::*base][base:1]]
pub struct Backtracking {
    epsilon: f64,
}

impl Default for Backtracking {
    fn default() -> Self {
        Backtracking {
            epsilon: 1e-3,
        }
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
