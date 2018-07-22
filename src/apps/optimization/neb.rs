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
    /// initial state
    pub mol1: Option<Molecule>,
    /// final state
    pub mol2: Option<Molecule>,
    /// NEB images: A list of Molecules without the two end points
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
            mol1   : None,
            mol2   : None,
        }
    }
}

impl NEB {
    pub fn new(mol1: Molecule, mol2: Molecule) -> Self {
        NEB {
            mol1: Some(mol1),
            mol2: Some(mol2),
            ..Default::default()
        }
    }

    // sanity check of images
    fn check_images(&self) -> Result<()>{
        let images = &self.images;
        if ! images.is_empty() {
            // 1. have the same number of atoms
            let mol = &images[0];
            let natoms = mol.natoms();
            for i in 0..images.len() {
                if images[i].natoms() != natoms {
                    bail!("mol {} has different number of atoms.", i);
                }
            }

            // 2. all images have the same order of elements
            let syms = mol.symbols();
            for i in 0..images.len() {
                let symsi = images[i].symbols();
                for j in 0..natoms {
                    if syms[j] != symsi[j] {
                        bail!("mol {}: incorrect order of element {}", i, symsi[j])
                    }
                }
            }
        } else {
            bail!("no images!");
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

        if let Some(ref mol1) = self.mol1 {
            if let Some(ref mol2) = self.mol2 {
                let tangents = tangent_vectors_original(mol1, mol2, &self.images)?;
                Ok(tangents)
            } else {
                bail!("no molecule (mol2) as the final state");
            }
        } else {
            bail!("no molecule (mol1) as the initial state");
        }
    }
}

// original algorithm for tangent calculation
// Ref: Classical and Quantum Dynamics in Condensed Phase Simulations; World Scientific, 1998; p 385.
fn tangent_vectors_original
    (
        mol1: &Molecule,
        mol2: &Molecule,
        images: &Vec<Molecule>
    ) -> Result<Vec<Positions3D>>
{
    let nmols = images.len();
    // tangent vectors along the path
    let mut tangents = Vec::with_capacity(nmols);
    // first_image - initial_state
    let pi = mol1.positions();
    let pj = images[0].positions();

    // normalized displacement vectors
    let mut disp_prev = get_normalized_displacements_between(&pj, &pi);

    // for intermediate images: between neighboring images
    for i in 0..(nmols-1) {
        let j = i + 1;
        let pi = images[i].positions();
        let pj = images[j].positions();

        // normalized displacement vectors
        let disp_next = get_normalized_displacements_between(&pj, &pi);
        let disp = &disp_next + &disp_prev;
        tangents.push(disp);
        disp_prev = disp_next;
    }

    // final_state - last_image
    let pj = mol2.positions();
    let pi = images[nmols-1].positions();
    let disp_next = get_normalized_displacements_between(&pj, &pi);
    tangents.push(disp_next + disp_prev);

    Ok(tangents)
}

// Henkelman, G.; Jónsson, H. J. Chem. Phys. 2000, 113 (22), 9978–9985.
fn tangent_vectors_improved
    (
        mol1: &Molecule,
        mol2: &Molecule,
        images: &Vec<Molecule>
    ) -> Result<Vec<Positions3D>>
{
    unimplemented!()
}

fn tangent_vectors_elastic_band
    (
        mol1: &Molecule,
        mol2: &Molecule,
        images: &Vec<Molecule>
    ) -> Result<Vec<Positions3D>>
{
    unimplemented!()
}
// 68a74344-0730-42bf-aa81-0c9405355dd1 ends here

// [[file:~/Workspace/Programming/gosh/gosh.note::755896f3-48b3-47b2-aa73-25f73a8b4b9a][755896f3-48b3-47b2-aa73-25f73a8b4b9a]]
#[test]
fn test_neb() {
    use gchemol::io;
    let mut images = io::read("tests/files/NEB/images.mol2").expect("neb test file");

    let mut neb = NEB::default();
    let mol1 = images.remove(0);
    let mol2 = images.pop().unwrap();
    let mut neb = NEB::new(mol1, mol2);
    neb.images = images;
    let ts = neb.tangent().unwrap();
    for x in ts {
        println!("{:}", x);
    }
}
// 755896f3-48b3-47b2-aa73-25f73a8b4b9a ends here
