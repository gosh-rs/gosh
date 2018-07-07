// [[file:~/Workspace/Programming/gosh/gosh.note::bbd2c8e8-5b09-4016-84ed-fc0f79a46c7f][bbd2c8e8-5b09-4016-84ed-fc0f79a46c7f]]
use super::*;

#[derive(Clone, Copy, Debug)]
pub struct LennardJones {
    /// Energy constant of the Lennard-Jones potential
    pub epsilon: f64,
    /// Distance constant of the Lennard-Jones potential
    pub sigma: f64,
}

impl Default for LennardJones {
    fn default() -> Self {
        LennardJones {
            epsilon: 1.0,
            sigma: 1.0,
        }
    }
}

impl LennardJones {
    fn pair_energy(&self, r: f64) -> f64 {
        let s6 = f64::powi(self.sigma / r, 6);
        4.0 * self.epsilon * (f64::powi(s6, 2) - s6)
    }

    fn pair_gradient(&self, r: f64) -> [f64; 3] {
        let s6 = f64::powi(self.sigma / r, 6);
        let g = 24.0 * self.epsilon * (s6 - 2.0 * f64::powi(s6, 2)) / r / r;

        [g, g, g]
    }
}

impl ChemicalModel for LennardJones {
    fn calculate(&self, mol: &Molecule) -> Result<ModelResults> {
        let mut energy = 0.0;

        let dm = mol.distance_matrix();
        for i in 0..mol.natoms() {
            for j in 0..i {
                let r = dm[i][j];
                energy += self.pair_energy(r);
            }
        }

        let mut mr = ModelResults::default();
        mr.energy = Some(energy);

        Ok(mr)
    }
}

#[test]
fn test_lj_model() {
    let lj = LennardJones::default();

    // LJ3
    let mol = Molecule::from_file("tests/files/LennardJones/LJ3.xyz").expect("lj3 test file");
    let mr = lj.calculate(&mol).expect("lj model: LJ3");
    let e = mr.energy.expect("lj model energy: LJ3");
    assert_relative_eq!(-3.0, e, epsilon=1e-3);
    // LJ38
    let mol = Molecule::from_file("tests/files/LennardJones/LJ38.xyz").expect("lj38 test file");
    let mr = lj.calculate(&mol).expect("lj model: LJ38");
    let e = mr.energy.expect("lj model energy: LJ38");
    assert_relative_eq!(-173.92843, e, epsilon=1e-3);
}
// bbd2c8e8-5b09-4016-84ed-fc0f79a46c7f ends here
