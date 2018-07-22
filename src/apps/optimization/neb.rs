// [[file:~/Workspace/Programming/gosh/gosh.note::b805a230-8b97-4b2e-ac45-0430598e1af8][b805a230-8b97-4b2e-ac45-0430598e1af8]]
//! Implementation of the Nudged Elastic Band (NEB) method for finding minimum energy paths and saddle points
//!
//! References
//! ----------
//! * Henkelman & Jonsson, JCP (113), 2000
//! * Henkelman, Uberuaga, & Jonsson, JCP (113), 2000

use super::*;
use gchemol::Molecule;
use gchemol::geometry::*;

#[derive(Debug, Clone)]
pub struct NEB {
    /// NEB images: A list of Molecules including the two end points
    images : Vec<Molecule>,
    /// climbing image or not
    climb  : bool,
    // FIXME: different k
    /// Spring constant
    k      : f64,
}

impl Default for NEB {
    fn default() -> Self {
        NEB {
            images : vec![],
            climb  : false,
            k      : 0.1,
        }
    }
}

impl NEB {
    pub fn new(images: Vec<Molecule>) -> Self {
        NEB {
            images,
            ..Default::default()
        }
    }

    // sanity check of images
    fn check_images(&self) -> Result<()>{
        let images = &self.images;
        if images.len() > 2 {
            // 1. have the same number of atoms
            let mol = &images[0];
            let natoms = mol.natoms();
            for i in 1..images.len() {
                if images[i].natoms() != natoms {
                    bail!("mol {} has different number of atoms.", i);
                }
            }

            // 2. all images have the same order of elements
            let syms = mol.symbols();
            for i in 1..images.len() {
                let symsi = images[i].symbols();
                for j in 0..natoms {
                    if syms[j] != symsi[j] {
                        bail!("mol {}: incorrect order of element {}", i, symsi[j])
                    }
                }
            }
        } else {
            bail!("not enough images!");
        }

        Ok(())
    }
}
// b805a230-8b97-4b2e-ac45-0430598e1af8 ends here

// [[file:~/Workspace/Programming/gosh/gosh.note::e40a0225-7988-43a1-ba61-adf6949d9d43][e40a0225-7988-43a1-ba61-adf6949d9d43]]
use nalgebra as na;
type Vector3D = na::Vector3<f64>;
// 3xN matrix storing 3D coordinates
type Positions3D = na::Matrix<f64, na::U3, na::Dynamic, na::MatrixVec<f64, na::U3, na::Dynamic>>;

fn get_positions_matrix(positions: &Vec<Point3D>) -> Positions3D {
    let pts: Vec<_> = positions.iter().map(|v| Vector3D::from(*v)).collect();
    Positions3D::from_columns(&pts)
}

// Return displacement matrix from positions2 to positions1
fn get_displacements_between(positions1: &Vec<Point3D>, positions2: &Vec<Point3D>) -> Positions3D {
    let npts = positions1.len();
    debug_assert!(npts == positions2.len());

    let pts1 = get_positions_matrix(positions1);
    let pts2 = get_positions_matrix(positions2);

    pts2 - pts1
}


// Return normalized displacement matrix from positions2 to positions1
fn get_normalized_displacements_between(positions1: &Vec<Point3D>, positions2: &Vec<Point3D>) -> Positions3D {
    let mut disp = get_displacements_between(positions1, positions2);
    let n = disp.normalize_mut();

    debug_assert!(n > 0.0, "normalized_disps: bad norm");

    disp
}
// e40a0225-7988-43a1-ba61-adf6949d9d43 ends here

// [[file:~/Workspace/Programming/gosh/gosh.note::68a74344-0730-42bf-aa81-0c9405355dd1][68a74344-0730-42bf-aa81-0c9405355dd1]]
impl NEB {
    fn tangent(&self) -> Result<Vec<Positions3D>> {
        // sanity check
        self.check_images()?;

        let tangents = tangent_vectors_original(&self.images)?;
        Ok(tangents)
    }
}
// 68a74344-0730-42bf-aa81-0c9405355dd1 ends here

// [[file:~/Workspace/Programming/gosh/gosh.note::d915c7b2-5fb5-4c26-8bde-baa6cfae3db9][d915c7b2-5fb5-4c26-8bde-baa6cfae3db9]]
// original algorithm for tangent calculation
// Ref: Classical and Quantum Dynamics in Condensed Phase Simulations; World Scientific, 1998; p 385.
fn tangent_vectors_original(images: &Vec<Molecule>) -> Result<Vec<Positions3D>>
{
    let nmols = images.len();
    assert!(nmols >= 3, "neb tangent original: not enough images");

    // tangent vectors along the path
    let mut tangents = Vec::with_capacity(nmols - 2 );

    // for intermediate images: between neighboring images
    for i in 1..(nmols-1) {
        let positions_this = images[i].positions();
        let positions_next = images[i+1].positions();
        let positions_prev = images[i-1].positions();

        // normalized displacement vectors
        let disp_next = get_normalized_displacements_between(&positions_next, &positions_this);
        let disp_prev = get_normalized_displacements_between(&positions_this, &positions_prev);
        let tangent = &disp_next + &disp_prev;
        tangents.push(tangent);
    }

    Ok(tangents)
}
// d915c7b2-5fb5-4c26-8bde-baa6cfae3db9 ends here

// [[file:~/Workspace/Programming/gosh/gosh.note::c6fdc171-9b1c-4ba8-a7c0-d3dbf57237eb][c6fdc171-9b1c-4ba8-a7c0-d3dbf57237eb]]
// Parameters
// ----------
// images: intermediate states
// energies: energies for the intermediate states
//
// Reference
// ---------
// Henkelman, G.; Jónsson, H. J. Chem. Phys. 2000, 113 (22), 9978–9985.
//
fn tangent_vectors_improved
    (
        images: &Vec<Molecule>,
        energies: &Vec<f64>,
    ) -> Result<Vec<Positions3D>>
{
    let nmols = images.len();
    debug_assert!(nmols == energies.len(), "neb improved: size different");
    debug_assert!(nmols >= 3, "neb improved: no intermediate image!");

    // tangent vectors along the path
    let mut tangents = Vec::with_capacity(nmols - 2);

    // loop over intermediate images excluding endpoints
    for i in 1..(nmols-1) {
        let positions_this = images[i].positions();
        let positions_next = images[i+1].positions();
        let positions_prev = images[i-1].positions();

        // displacement vectors
        let disp_next = get_displacements_between(&positions_next, &positions_this);
        let disp_prev = get_displacements_between(&positions_this, &positions_prev);
        let energy_this = energies[i];
        let energy_next = energies[i+1];
        let energy_prev = energies[i-1];

        let mut tangent = {
            if energy_next > energy_this && energy_this > energy_prev {
                disp_next
            } else if energy_next < energy_this && energy_this < energy_prev {
                disp_prev
            } else {
                let d1 = (energy_next - energy_this).abs();
                let d2 = (energy_this - energy_prev).abs();
                let delta_energy_max = d1.max(d2);
                let delta_energy_min = d1.min(d2);
                if energy_next > energy_prev {
                    disp_next * delta_energy_max + disp_prev * delta_energy_min
                } else {
                    disp_next * delta_energy_min + disp_prev * delta_energy_max
                }
            }
        };
        let n = tangent.normalize_mut();
        assert!(n.is_sign_positive(), "neb improved tangent: weird norm");

        tangents.push(tangent);
    }

    Ok(tangents)
}
// c6fdc171-9b1c-4ba8-a7c0-d3dbf57237eb ends here

// [[file:~/Workspace/Programming/gosh/gosh.note::82c2f7d1-cec4-4866-a9ba-7be37b872a95][82c2f7d1-cec4-4866-a9ba-7be37b872a95]]
fn tangent_vectors_elastic_band
    (
        mol1: &Molecule,
        mol2: &Molecule,
        images: &Vec<Molecule>
    ) -> Result<Vec<Positions3D>>
{
    unimplemented!()
}
// 82c2f7d1-cec4-4866-a9ba-7be37b872a95 ends here

// [[file:~/Workspace/Programming/gosh/gosh.note::755896f3-48b3-47b2-aa73-25f73a8b4b9a][755896f3-48b3-47b2-aa73-25f73a8b4b9a]]
#[test]
fn test_neb() {
    use gchemol::io;
    let mut images = io::read("tests/files/NEB/images.mol2").expect("neb test file");

    let mut neb = NEB::new(images);
    let ts = neb.tangent().unwrap();
    for x in ts {
        println!("{:}", x);
    }
}
// 755896f3-48b3-47b2-aa73-25f73a8b4b9a ends here
