// [[file:~/Workspace/Programming/gosh/gosh.note::fdd9627f-71a8-4a4f-b0a7-9f1d2af71da3][fdd9627f-71a8-4a4f-b0a7-9f1d2af71da3]]
use super::*;

pub struct FIRE {
    /// adaptive time step for integration of the equations of motion
    dt: f64,
    /// the maximum time step permitted
    dtmax: f64,
    /// adaptive parameter that controls the velocity used to evolve the system.
    alpha: f64,
    /// the maximum step size permitted
    maxmove: f64,
    /// factor by which the time step is decreased if an uphill step has been taken
    fdec: f64,
    /// factor by which parameter a decreases if moving downhill and at least
    /// nmin steps from the last uphill step
    fa: f64,

    velocities: Option<Vec<[f64; 3]>>,
    nsteps: usize,
    /// minimum number of steps taken after an uphill step before adaptive
    /// parameters are allowed to change
    nmin: usize,
}

impl Default for FIRE {
    fn default() -> Self {
        FIRE {
            dt: 0.1,
            dtmax: 1.0,
            alpha: 0.10,
            fdec: 0.50,
            fa: 0.99,
            maxmove: 0.5,

            velocities: None,
            nsteps: 0,
            nmin: 5,
        }
    }
}

// V = (1 - alpha) · V + alpha · F / |F| · |V|
fn update_velocities(velocities: &mut Vec<[f64; 3]>, forces: &Vec<[f64; 3]>, alpha: f64) {
    let n = velocities.len();
    let vnorm = vdot(&velocities, &velocities).sqrt();
    let fnorm = vdot(&forces, &forces).sqrt();
    for i in 0..n {
        for j in 0..3 {
            let fij = forces[i][j];
            velocities[i][j] *= (1.0 - alpha);
            velocities[i][j] += alpha * fij * vnorm / fnorm;
        }
    }
}

impl FIRE {
    fn init_velocities(&mut self, natoms: usize) {
        // initialize velocities with zeros
        let mut velocities = Vec::with_capacity(natoms);
        for _ in 0..natoms {
            velocities.push([0.0; 3]);
        }
        self.velocities = Some(velocities);
    }

    /// update positions using FIRE algorithm
    pub fn next_step(&mut self, forces: &Vec<[f64; 3]>) -> Result<()> {
        let natoms = forces.len();
        if self.velocities.is_none() {
            &mut self.init_velocities(natoms);
        }

        if let Some(ref mut velocities) = &mut self.velocities {
            // P = F·V
            let p = vdot(&forces, &velocities);

            // check direction
            if p.is_sign_positive() {
                update_velocities(velocities, &forces, self.alpha);
                if self.nsteps > self.nmin {
                    self.dt *= self.fdec;
                    if self.dt > self.dtmax {
                        self.dt = self.dtmax;
                    }
                    self.alpha *= self.fa;
                }
                self.nsteps += 1;
            } else {
                // reset alpha
                self.alpha = 0.1;
                self.dt *= self.fdec;
                self.nsteps = 0;

                // reset velocities to zero
                for i in 0..natoms {
                    for j in 0..3 {
                        velocities[i][j] = 0.0;
                    }
                }
            }

            // calculate positions shift
            let mut positions_delta = Vec::with_capacity(natoms);
            for i in 0..natoms {
                let mut position = [0.0; 3];
                for j in 0..3 {
                    let fij = forces[i][j];
                    velocities[i][j] += self.dt * fij;
                    position[j] = self.dt * velocities[i][j]
                }
                positions_delta.push(position);
            }

            let dr_norm = vdot(&positions_delta, &positions_delta).sqrt();
            if dr_norm > self.maxmove {
                let scale = self.maxmove / dr_norm;
                for i in 0..natoms {
                    for j in 0..3 {
                        positions_delta[i][j] *= scale;
                    }
                }
            }
        }

        Ok(())
    }
}

#[inline]
fn vdot(vector1: &Vec<[f64; 3]>, vector2: &Vec<[f64; 3]>) -> f64 {
    let n = vector1.len();
    debug_assert!(n == vector2.len());

    let mut vret = 0.0;
    for i in 0..n {
        for j in 0..3 {
            let vij1 = vector1[i][j];
            let vij2 = vector2[i][j];
            vret += vij1 * vij2;
        }
    }

    vret
}

#[test]
fn test_vdot() {
    let a = vec![
        [1., 4., 0.0],
        [5., 6., 0.0]
    ];

    let b = vec![
        [4., 1., 0.0],
        [2., 2., 0.0]
    ];

    let x = vdot(&a, &b);
    assert_relative_eq!(30.0, x, epsilon=1e-4);
}
// fdd9627f-71a8-4a4f-b0a7-9f1d2af71da3 ends here
