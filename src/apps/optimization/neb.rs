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

/// A single image in the chain of NEB images.
#[derive(Debug, Clone)]
pub struct Image {
    /// internal molecule
    pub mol    : Molecule,
    /// spring constant
    pub k      : f64,
    /// real energy
    pub energy : Option<f64>,
    /// real forces
    pub forces : Option<Positions3D>,
}

impl Image {
    /// Construct image from molecule
    pub fn new(mol: Molecule) -> Self {
        Image {
            mol,
            k: 0.1,
            energy: None,
            forces: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NEB {
    /// NEB images: A list of Molecules including the two end points
    pub images : Vec<Image>,
    /// climbing image or not
    pub climbing  : bool,
}

impl Default for NEB {
    fn default() -> Self {
        NEB {
            images : vec![],
            climbing  : false,
        }
    }
}

impl NEB {
    pub fn new(mols: Vec<Molecule>) -> Self {
        // build up images from molecules
        let mut images = Vec::with_capacity(mols.len());
        for mol in mols {
            let image = Image::new(mol);
            images.push(image);
        }

        NEB {
            images,
            ..Default::default()
        }
    }

    // sanity check of images
    fn check_images(&self) -> Result<()>{
        let images = &self.images;

        let nimages = images.len();
        // 0. required at least 3 images
        if nimages > 2 {
            // 1. molecule for each image has the same number of atoms
            let mol = &images[0].mol;
            let natoms = mol.natoms();
            for i in 1..nimages {
                if images[i].mol.natoms() != natoms {
                    bail!("molecule in image {} has different number of atoms.", i);
                }
            }

            // 2. molecule for each image shares the same order of elements
            let syms = mol.symbols();
            for i in 1..nimages {
                let symsi = images[i].mol.symbols();
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

// Return displacement vectors: positions_next - positions_this
fn get_displacements_between(positions_next: &Vec<Point3D>, positions_this: &Vec<Point3D>) -> Positions3D {
    debug_assert!(positions_this.len() == positions_next.len());

    let pts1 = get_positions_matrix(positions_this);
    let pts2 = get_positions_matrix(positions_next);

    pts2 - pts1
}

// Return displacement vectors between every pair of neighboring images
// displ = R_{i+1} - R_{i}
fn get_neighboring_images_displacements(images: &Vec<Image>) -> Result<Vec<Positions3D>>
{
    let nmols = images.len();
    assert!(nmols >= 3, "neb tangent original: not enough images");

    // tangent vectors along the path
    let mut displs = Vec::with_capacity(nmols - 1);

    // for intermediate images: between neighboring images
    for i in 0..(nmols-1) {
        let positions_this = images[i].mol.positions();
        let positions_next = images[i+1].mol.positions();

        // normalized displacement vectors
        let displ = get_displacements_between(&positions_next, &positions_this);
        displs.push(displ);
    }

    Ok(displs)
}
// e40a0225-7988-43a1-ba61-adf6949d9d43 ends here

// [[file:~/Workspace/Programming/gosh/gosh.note::68a74344-0730-42bf-aa81-0c9405355dd1][68a74344-0730-42bf-aa81-0c9405355dd1]]
impl NEB {
    fn tangents(&self) -> Result<Vec<Positions3D>> {
        // sanity check
        self.check_images()?;

        let displacements = get_neighboring_images_displacements(&self.images)?;
        let tangents = tangent_vectors_original(&displacements)?;
        Ok(tangents)
    }

    /// calculate real energy and forces
    fn calculate<T: ChemicalModel>(&mut self, model: T) -> Result<()>{
        let nimages = self.images.len();
        // FIXME: special treatment for initial state and final state
        // calculate image energies and forces
        for i in 0..nimages {
            // 0. run the model
            let mr = {
                let mol = &self.images[i].mol;
                model.calculate(mol)?
            };

            // 1. get the energy
            if let Some(energy) = mr.energy {
                self.images[i].energy = Some(energy);
            } else {
                bail!("no energy");
            }

            // 2. get the forces
            if let Some(forces) = mr.forces {
                // FIXME: need util in gchemol
                let fm = get_positions_matrix(&forces);
                self.images[i].forces = Some(fm);
            } else {
                bail!("no forces");
            }
        }

        Ok(())
    }

    /// Return the resulting NEB forces for all images
    fn neb_forces(&self) -> Result<Vec<Positions3D>> {
        // calculate image tangents
        let tangents = self.tangents()?;
        // let f1 = spring_forces_parallel();
        // let f2 = real_forces_perpendicular();
        // let fneb = f1 + f2;
        unimplemented!()
    }
}

// Calculate the parallel component of the spring force
fn spring_forces_parallel(displacements: &Vec<Positions3D>,
                          spring_constants: &Vec<f64>,
                          tangents: &Vec<Positions3D>) -> Vec<Positions3D>
{
    let nmols = tangents.len() + 2;
    debug_assert!(nmols - 1 == displacements.len());
    debug_assert!(nmols - 2 == spring_constants.len());

    let mut forces = Vec::with_capacity(nmols - 2);
    // calculate spring forces for all intermediate images
    // loop over intermediate images excluding two endpoints
    for i in 1..(nmols-1) {
        // displacement vectors: R_{i} - R_{i-1}
        let displ_prev = &displacements[i-1];
        // displacement vectors: R_{i+1} - R_{i}
        let displ_next = &displacements[i];
        // spring constant of the previous pair
        let kprev = spring_constants[i-1];
        // spring constant of the next pair
        let knext = spring_constants[i];
        // tangent vector for current image
        let tangent = &tangents[i];
        let f = (displ_next.norm() * knext - displ_prev.norm() * kprev) * tangent;
        forces.push(f);
    }
    forces
}

// Calculate the perpendicular component of the real forces
// Parameters
// ----------
// all_forces: forces of molecules in all intermediate images
// tangents: tangent vectors of all intermediate images
fn real_forces_perpendicular(all_forces: &Vec<Positions3D>,
                             tangents: &Vec<Positions3D>) -> Vec<Positions3D> {

    let n = all_forces.len();
    debug_assert!(n == tangents.len());

    let nmols = n + 2;

    let mut vforces = Vec::with_capacity(nmols - 2);
    for i in 1..(nmols - 2) {
        let fi = &all_forces[i];
        let ti = &tangents[i];
        let f = fi - fi * ti * ti;
        vforces.push(f);
    }

    vforces
}
// 68a74344-0730-42bf-aa81-0c9405355dd1 ends here

// [[file:~/Workspace/Programming/gosh/gosh.note::d915c7b2-5fb5-4c26-8bde-baa6cfae3db9][d915c7b2-5fb5-4c26-8bde-baa6cfae3db9]]
// original algorithm for tangent calculation
// Ref: Classical and Quantum Dynamics in Condensed Phase Simulations; World Scientific, 1998; p 385.
fn tangent_vectors_original(displacements: &Vec<Positions3D>) -> Result<Vec<Positions3D>>
{
    let nmols = displacements.len() + 1;
    assert!(nmols >= 3, "neb tangent original: not enough images");

    // tangent vectors along the path
    let mut tangents = Vec::with_capacity(nmols - 2 );

    // for intermediate images: between neighboring images
    for i in 1..(nmols-1) {
        // normalized R_{i+1} - R_{i}
        let disp_next = displacements[i].normalize();
        // normalized R_{i} - R_{i-1}
        let disp_prev = displacements[i-1].normalize();
        let tangent = disp_next + disp_prev;
        tangents.push(tangent);
    }

    Ok(tangents)
}
// d915c7b2-5fb5-4c26-8bde-baa6cfae3db9 ends here

// [[file:~/Workspace/Programming/gosh/gosh.note::c6fdc171-9b1c-4ba8-a7c0-d3dbf57237eb][c6fdc171-9b1c-4ba8-a7c0-d3dbf57237eb]]
// Parameters
// ----------
// displacements: displacement vectors between neighboring images
// energies: energies of molecules in the images
//
// Reference
// ---------
// Henkelman, G.; Jónsson, H. J. Chem. Phys. 2000, 113 (22), 9978–9985.
//
fn tangent_vectors_improved
    (
        displacements: &Vec<Positions3D>,
        energies: &Vec<f64>,
    ) -> Result<Vec<Positions3D>>
{
    let nmols = energies.len();
    debug_assert!(nmols >= 3, "neb improved: no intermediate image!");
    debug_assert!(nmols == displacements.len() + 1, "neb improved: size different");

    // tangent vectors for intermediate images
    let mut tangents = Vec::with_capacity(nmols - 2);

    // loop over intermediate images excluding endpoints
    for i in 1..(nmols-1) {
        // displacement vectors: R_{i} - R_{i-1}
        let ref disp_prev = displacements[i-1];
        // displacement vectors: R_{i+1} - R_{i}
        let ref disp_next = displacements[i];
        let energy_prev = energies[i-1];
        let energy_this = energies[i];
        let energy_next = energies[i+1];

        let mut tangent = {
            if energy_next > energy_this && energy_this > energy_prev {
                disp_next * 1.0 // return a copy
            } else if energy_next < energy_this && energy_this < energy_prev {
                disp_prev * 1.0 // return a copy
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
        assert!(n > 1e-3, "neb improved tangent: weird norm");
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
    let ts = neb.tangents().unwrap();
    for x in ts {
        println!("{:}", x);
    }
}
// 755896f3-48b3-47b2-aa73-25f73a8b4b9a ends here
