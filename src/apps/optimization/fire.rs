// [[file:~/Workspace/Programming/gosh/gosh.note::4aa0086b-0cf4-406a-861a-b281b328ef2e][4aa0086b-0cf4-406a-861a-b281b328ef2e]]
/// Implementation of the Fast-Inertial-Relaxation-Engine (FIRE) algorithm
///
/// References
/// ----------
/// (1) Bitzek, E. et al. Structural Relaxation Made Simple. Phys. Rev. Lett. 2006, 97 (17), 170201.
/// (2) http://users.jyu.fi/~pekkosk/resources/pdf/FIRE.pdf
/// (3) https://github.com/siesta-project/flos/blob/master/flos/optima/fire.lua

use super::*;

#[derive(Debug, Clone)]
pub struct FIRE {
    /// the maximum time step allowed
    dt_max: f64,
    /// factor used to decrease alpha-parameter if downhill
    f_alpha: f64,
    /// initial alpha-parameter
    alpha_start: f64,
    /// the maximum displacement allowed
    maxdisp: f64,
    /// factor used to increase time-step if downhill
    f_inc: f64,
    /// factor used to decrease time-step if uphill
    f_dec: f64,
    /// minimum number of iterations ("latency" time) performed before time-step
    /// may be increased, which is important for the stability of the algorithm.
    nsteps_min: usize,

    /// adaptive time step for integration of the equations of motion
    dt: f64,
    /// adaptive parameter that controls the velocity used to evolve the system.
    alpha: f64,
    /// internal current velocities
    velocities: Option<Vec<[f64; 3]>>,
    /// current number of iterations when go downhill
    nsteps: usize,
}

impl Default for FIRE {
    fn default() -> Self {
        FIRE {
            // default parameters taken from the original paper
            dt_max     : 1.00,
            alpha_start: 0.10,
            f_alpha    : 0.99,
            f_dec      : 0.50,
            f_inc      : 1.10,
            maxdisp    : 0.50,
            nsteps_min : 5,

            // counters or adaptive parameters
            dt         : 0.10,
            alpha      : 0.10,
            nsteps     : 0,
            velocities : None,
        }
    }
}
// 4aa0086b-0cf4-406a-861a-b281b328ef2e ends here

// [[file:~/Workspace/Programming/gosh/gosh.note::fdd9627f-71a8-4a4f-b0a7-9f1d2af71da3][fdd9627f-71a8-4a4f-b0a7-9f1d2af71da3]]
type Point3D = [f64; 3];

impl FIRE {
    /// Determine whether we have optimized the structure
    pub fn converged(&self, forces: &Vec<Point3D>, displacement_vectors: &Vec<Point3D>) -> bool {
        debug_assert!(forces.len() == displacement_vectors.len(), "vectors in different size");
        let fnorms = vector_norms(&forces);
        let dnorms = vector_norms(displacement_vectors);

        // FIXME: criteria parameters
        let fmax = 0.01;
        let dmax = 0.02;
        let fcur = fnorms.max();
        let dcur = dnorms.max();
        println!("{:#?}", (fcur, dcur));
        if fcur < fmax && dcur < dmax {
            true
        } else {
            false
        }
    }

    /// get displacement vectors for all atoms
    pub fn displacement_vectors(&mut self, forces: &Vec<Point3D>) -> Result<Vec<Point3D>> {
        let natoms = forces.len();
        let velocities = self.velocities.take();
        if let Some(mut velocities) = velocities {
            let r = self.propagate(&forces, &mut velocities);
            self.velocities = Some(velocities);
            r
        } else {
            let mut velocities = zero_velocities(natoms);
            let r = self.propagate(&forces, &mut velocities);
            self.velocities = Some(velocities);
            r
        }
    }

    /// Propagate the system for one simulation step using FIRE algorithm.
    fn propagate(&mut self, forces: &Vec<Point3D>, velocities: &mut Vec<Point3D>) -> Result<Vec<Point3D>> {
        // F1. calculate the power: P = F·V
        let power = vector_dot(&forces, &velocities);

        // F2. adjust velocities
        update_velocities(velocities, &forces, self.alpha);

        // F3 & F4: check the direction of power: go downhill or go uphill
        if power.is_sign_positive() {
            // F3. when go downhill
            // increase time step if we have go downhill for enough times
            if self.nsteps > self.nsteps_min {
                self.dt *= self.f_inc;
                if self.dt > self.dt_max {
                    self.dt = self.dt_max;
                }
                self.alpha *= self.f_alpha;
            }
            // increment step counter
            self.nsteps += 1;
        } else {
            // F4. when go uphill
            // decrease time-step and reset alpha
            self.dt *= self.f_dec;
            self.alpha = self.alpha_start;
            // reset velocities to zero
            let natoms = forces.len();
            for i in 0..natoms {
                for j in 0..3 {
                    velocities[i][j] = 0.0;
                }
            }
            // reset step counter
            self.nsteps = 0;
        }

        // F5. calculate displacement vectors based on a typical MD stepping algorithm
        let mut disp_vectors = get_md_displacement_vectors(&forces, &velocities, self.alpha);

        // scale the displacement according to max displacement
        scale_disp_vectors(&mut disp_vectors, self.maxdisp);

        // update the internal velocities
        for i in 0..forces.len() {
            for j in 0..3 {
                velocities[i][j] += forces[i][j] * self.dt;
            }
        }

        Ok(disp_vectors)
    }
}

fn zero_velocities(natoms: usize) -> Vec<Point3D> {
    // initialize velocities with zeros
    let mut velocities = Vec::with_capacity(natoms);
    for _ in 0..natoms {
        velocities.push([0.0; 3]);
    }
    velocities
}

// get particle displacement vectors by performing a regular MD step
fn get_md_displacement_vectors
    (
        forces     : &Vec<Point3D>,
        velocities : &Vec<Point3D>,
        timestep   : f64
    ) -> Vec<Point3D>
{
    let natoms = velocities.len();
    debug_assert!(natoms == forces.len(), "input vectors are in different size!");

    // Verlet algorithm
    let mut disp_vectors = Vec::with_capacity(natoms);
    let dt = timestep;
    for i in 0..natoms {
        let mut position = [0.0; 3];
        for j in 0..3 {
            let fij = forces[i][j];
            let vij = velocities[i][j];
            position[j] = dt * vij + 0.5 * fij * dt * dt;
        }
        disp_vectors.push(position);
    }

    disp_vectors
}

// Update velocities
// V = (1 - alpha) · V + alpha · F / |F| · |V|
fn update_velocities(velocities: &mut Vec<Point3D>, forces: &Vec<Point3D>, alpha: f64) {
    let n = velocities.len();
    let vnorm = vector_dot(&velocities, &velocities).sqrt();
    let fnorm = vector_dot(&forces, &forces).sqrt();
    for i in 0..n {
        for j in 0..3 {
            let fij = forces[i][j];
            velocities[i][j] *= 1.0 - alpha;
            velocities[i][j] += alpha * fij * vnorm / fnorm;
        }
    }
}

// scale the displacement vectors if exceed a given max displacement.
fn scale_disp_vectors(disp_vectors: &mut Vec<Point3D>, maxdisp: f64) {
    // get the max norm of displacement vector for atoms
    let mut norm_max = 0.0;
    for i in 0..disp_vectors.len() {
        let mut d = 0.0;
        for j in 0..3 {
            let dij = disp_vectors[i][j];
            d += dij.powi(2);
        }
        let d = d.sqrt();
        if d > norm_max {
            norm_max = d;
        }
    }

    // scale the displacement vectors if too large
    let natoms = disp_vectors.len();
    if norm_max > maxdisp {
        let scale = maxdisp / norm_max;
        for i in 0..natoms {
            for j in 0..3 {
                disp_vectors[i][j] *= scale;
            }
        }
    }
}
// fdd9627f-71a8-4a4f-b0a7-9f1d2af71da3 ends here

// [[file:~/Workspace/Programming/gosh/gosh.note::ac85201d-3985-4160-886b-1f811e6db4b9][ac85201d-3985-4160-886b-1f811e6db4b9]]
use test::stats::Stats;

// return the norms of a list of 3D vectors
fn vector_norms(vectors: &Vec<Point3D>) -> Vec<f64> {
    let n = vectors.len();
    let mut norms = Vec::with_capacity(n);

    for i in 0..n {
        let mut l = 0.0;
        for j in 0..3 {
            let vij = vectors[i][j];
            l += vij.powi(2);
        }

        norms.push(l.sqrt());
    }

    norms
}

#[inline]
fn vector_dot(vector1: &Vec<[f64; 3]>, vector2: &Vec<[f64; 3]>) -> f64 {
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
fn test_vector_dot() {
    let a = vec![
        [1., 4., 0.0],
        [5., 6., 0.0]
    ];

    let b = vec![
        [4., 1., 0.0],
        [2., 2., 0.0]
    ];

    let x = vector_dot(&a, &b);
    assert_relative_eq!(30.0, x, epsilon=1e-4);
}
// ac85201d-3985-4160-886b-1f811e6db4b9 ends here

// [[file:~/Workspace/Programming/gosh/gosh.note::759e21a2-1f20-4c38-a7c4-9601c1281347][759e21a2-1f20-4c38-a7c4-9601c1281347]]
use gchemol::Molecule;

#[test]
fn test_fire_opt() {
    use models::ChemicalModel;
    use models::lj::LennardJones;
    let mut mol = get_test_mol();
    let mut lj = LennardJones::default();
    lj.derivative_order = 1;

    let mut fire = FIRE::default();
    let natoms = mol.natoms();
    for i in 0..500 {
        let mresult = lj.calculate(&mol).expect("lj calculation");
        let energy = mresult.energy.expect("lj energy");
        println!("step {}: energy = {:-6.3}", i, energy);

        let forces = mresult.forces.expect("lj forces");
        let dvects = fire.displacement_vectors(&forces).expect("dv");
        // update positions
        let mut positions = mol.positions();
        for j in 0..natoms {
            for k in 0..3 {
                positions[j][k] += dvects[j][k];
            }
        }

        if fire.converged(&forces, &dvects) {
            break;
        }

        mol.set_positions(positions);
    }
}

fn get_test_mol() -> Molecule {
    let txt = " 6
Title Card Required
C              -0.131944         -0.282942          0.315957
H               0.401220         -1.210646          0.315957
H              -1.201944         -0.282942          0.315957
C               0.543331          0.892036          0.315957
H               0.010167          1.819741          0.315957
H               1.613331          0.892036          0.315957
";
    Molecule::parse_from(txt, "text/xyz").expect("mol from xyz string")
}
// 759e21a2-1f20-4c38-a7c4-9601c1281347 ends here