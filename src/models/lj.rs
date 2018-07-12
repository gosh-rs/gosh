// [[file:~/Workspace/Programming/gosh/gosh.note::bbd2c8e8-5b09-4016-84ed-fc0f79a46c7f][bbd2c8e8-5b09-4016-84ed-fc0f79a46c7f]]
use super::*;

#[derive(Clone, Copy, Debug)]
pub struct LennardJones {
    /// Energy constant of the Lennard-Jones potential
    pub epsilon: f64,
    /// Distance constant of the Lennard-Jones potential
    pub sigma: f64,

    pub derivative_order: usize,
}

impl Default for LennardJones {
    fn default() -> Self {
        LennardJones {
            epsilon: 1.0,
            sigma: 1.0,
            // energy only
            derivative_order: 0,
        }
    }
}

impl LennardJones {
    // vij
    fn pair_energy(&self, r: f64) -> f64 {
        let s6 = f64::powi(self.sigma / r, 6);
        4.0 * self.epsilon * (f64::powi(s6, 2) - s6)
    }

    // dvij
    fn pair_gradient(&self, r: f64) -> f64 {
        let s6 = f64::powi(self.sigma / r, 6);

        24.0 * self.epsilon * (s6 - 2.0 * f64::powi(s6, 2)) / r
    }
}

impl ChemicalModel for LennardJones {
    fn calculate(&self, mol: &Molecule) -> Result<ModelResults> {
        if mol.lattice.is_some() {
            warn!("LJ model: periodic lattice will be ignored!")
        }

        let natoms = mol.natoms();
        let mut energy = 0.0;
        let mut forces = Vec::with_capacity(natoms);

        // initialize with zeros
        for i in 0..natoms {
            forces.push([0.0; 3]);
        }

        // calculate energy and forces
        let positions = mol.positions();
        let dm = mol.distance_matrix();
        for i in 0..natoms {
            for j in 0..i {
                let r = dm[i][j];
                energy += self.pair_energy(r);
                if self.derivative_order >= 1 {
                    let g = self.pair_gradient(r);
                    for k in 0..3 {
                        let dr = positions[j][k] - positions[i][k];
                        forces[i][k] += 1.0 * g * dr / r;
                        forces[j][k] += -1.0 * g * dr / r;
                    }
                }
            }
        }

        let mut mr = ModelResults::default();
        mr.energy = Some(energy);

        if self.derivative_order >= 1 {
            mr.forces = Some(forces);
        }
        if self.derivative_order >= 2 {
            unimplemented!();
        }

        Ok(mr)
    }
}

#[test]
fn test_lj_model() {
    let mut lj = LennardJones::default();
    lj.derivative_order = 1;

    // LJ3
    let mol = Molecule::from_file("tests/files/LennardJones/LJ3.xyz").expect("lj3 test file");
    let mr = lj.calculate(&mol).expect("lj model: LJ3");
    let e = mr.energy.expect("lj model energy: LJ3");
    assert_relative_eq!(-3.0, e, epsilon=1e-3);

    let forces = mr.forces.expect("lj model forces: LJ3");
    for i in 0..mol.natoms() {
        for j in 0..3 {
            assert_relative_eq!(0.0, forces[i][j], epsilon=1e-3);
        }
    }

    // LJ38
    let mol = Molecule::from_file("tests/files/LennardJones/LJ38.xyz").expect("lj38 test file");
    let mr = lj.calculate(&mol).expect("lj model: LJ38");
    let e = mr.energy.expect("lj model energy: LJ38");
    assert_relative_eq!(-173.92843, e, epsilon=1e-3);

    let forces = mr.forces.expect("lj model forces: LJ3");
    for i in 0..mol.natoms() {
        for j in 0..3 {
            assert_relative_eq!(0.0, forces[i][j], epsilon=1e-3);
        }
    }
}
// bbd2c8e8-5b09-4016-84ed-fc0f79a46c7f ends here
