// base

// [[file:~/Workspace/Programming/gosh/gosh.note::*base][base:1]]
//! Implementation of the Nudged Elastic Band (NEB) method for finding minimum energy paths and saddle points
//!
//! References
//! ----------
//! * Henkelman, G. et al J. Chem. Phys. 2000, 113 (22), 9978–9985.
//! * Henkelman, G. et al J. Chem. Phys. 2000, 113 (22), 9901–9904.
//! * Kolsbjerg et al J. Chem. Phys. 2016, 145 (9), 094107.
//! * https://github.com/siesta-project/flos/blob/master/flos/special/neb.lua

use super::*;
use gchemol::Molecule;

/// A single image in the chain of NEB chain.
#[derive(Debug, Clone)]
pub struct Image {
    /// internal molecule
    pub mol    : Molecule,
    /// real energy
    pub energy : Option<f64>,
    /// real forces
    pub forces : Option<Vector3fVec>,
}

impl Image {
    /// Construct image from molecule
    pub fn new(mol: Molecule) -> Self {
        Image {
            mol,
            energy  : None,
            forces  : None,
        }
    }
}

/// Nudged Elastic Band (NEB) method
#[derive(Debug, Clone)]
pub struct NEB {
    /// NEB images: A list of Molecules including the two end points
    pub images : Vec<Image>,
    /// Using climbing image or not
    pub climbing  : bool,
    /// Spring force constants between neighboring images
    pub spring_constants: Vec<f64>,
    /// the tolerance for determining whether an image is climbing or not
    climbing_tol: f64,
}

impl Default for NEB {
    fn default() -> Self {
        NEB {
            images           : vec![],
            climbing         : false,
            spring_constants : vec![],
            climbing_tol     : 0.005,
        }
    }
}

impl NEB {
    pub fn new(mols: Vec<Molecule>) -> Self {
        let n = mols.len();
        // build up images from molecules
        let mut images = Vec::with_capacity(mols.len());
        for mol in mols {
            let image = Image::new(mol);
            images.push(image);
        }

        // the number for springs
        let n = n - 1;
        NEB {
            images,
            spring_constants: (0..n).map(|v| 0.1).collect(),
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
// base:1 ends here

// core

// [[file:~/Workspace/Programming/gosh/gosh.note::*core][core:1]]
use crate::apps::optimization::fire::FIRE;

impl NEB {
    /// Carry out NEB optimization using a chemical model
    pub fn run<T: ChemicalModel>(&mut self, model: &T) -> Result<()> {
        let nimages = self.images.len();

        // optimization loop
        let mut fire = FIRE::default();
        for i in 0..200 {
            // 1. real calculation
            self.calculate(model)?;
            let arr_forces = self.neb_forces()?;
            println!("cycle {:}", i);
            let forces = forces_mat_to_vec(&arr_forces);
            let dvects = fire.displacement_vectors(&forces)?;
            if fire.converged(&forces, &dvects) {
                break;
            }

            // 2. update positions
            for i in 1..(nimages-1) {
                let mol = &mut self.images[i].mol;
                let mut positions = mol.positions();
                let natoms = mol.natoms();
                let shift = (i - 1) * natoms;
                for j in 0..natoms {
                    let x = shift + j;
                    for k in 0..3 {
                        positions[j][k] += dvects[x][k];
                    }
                }
                mol.set_positions(&positions)?;
            }
        }

        Ok(())
    }

    /// calculate real energy and forces
    fn calculate<T: ChemicalModel>(&mut self, model: &T) -> Result<()>{
        let nimages = self.images.len();
        // FIXME: special treatment for initial state and final state
        // calculate image energies and forces
        let mut mols = vec![];
        for image in &self.images {
            mols.push(image.mol.clone());
        }

        let arr_mp = model.compute_many(&mols)?;
        for (i, mp) in arr_mp.into_iter().enumerate() {
            // 0. run the model
            // 1. get the energy
            if let Some(energy) = mp.energy {
                self.images[i].energy = Some(energy);
            } else {
                bail!("no energy");
            }

            // 2. get the forces
            if let Some(forces) = mp.forces {
                let fm = forces.to_dmatrix();
                self.images[i].forces = Some(fm);
            } else {
                bail!("no forces");
            }
        }

        Ok(())
    }

    /// Return the resulting NEB forces of all images excluding endpoints
    fn neb_forces(&self) -> Result<Vec<Vector3fVec>> {
        // sanity check
        self.check_images()?;

        // 0. collect intermediate results
        let displacements = get_neighboring_images_displacements(&self.images)?;
        let (energies, forces) = self.collect_energies_and_forces()?;

        // 1. calculate image tangent vectors to NEB path
        // let tangents = tangent_vectors_original(&displacements)?;
        let tangents = tangent_vectors_improved(&displacements, &energies)?;

        // 2. the parallel part from spring forces
        let forces1 = spring_forces_parallel(&displacements, &self.spring_constants, &tangents);

        // 3. the perpendicular part from real forces
        let forces2 = real_forces_perpendicular(&forces, &tangents);

        // 4. neb forces
        let n = forces1.len();
        let mut forces_neb = Vec::with_capacity(n);
        for i in 0..n {
            let f = &forces1[i] + &forces2[i];
            forces_neb.push(f);
        }

        if self.climbing {
            self.fix_climbing_image(&mut forces_neb, &energies, &forces, &tangents)?;
        }

        Ok(forces_neb)
    }

    /// Fix forces for the climbing image
    /// Parameters
    /// ----------
    /// - forces_neb: calculated neb forces in regular way
    /// - energies: molecule energy in each image
    fn fix_climbing_image(&self,
                          forces_neb: &mut Vec<Vector3fVec>,
                          energies: &Vec<f64>,
                          forces: &Vec<Vector3fVec>,
                          tangents: &Vec<Vector3fVec>) -> Result<()>
    {
        let n = self.images.len();

        // energy tolerance for determing climbing images
        let tol = self.climbing_tol;

        // locate the climbing image
        let mut candidates = vec![];
        for i in 1..(n-1) {
            let eprev = energies[i-1];
            let ethis = energies[i  ];
            let enext = energies[i+1];

            if ethis - eprev > tol && ethis - enext > tol {
                candidates.push(i);
            }
        }

        // FIXME: how about this?
        if candidates.len() != 1 {
            bail!("found {} peaks for climbing..", candidates.len());
        }

        let imax = candidates.pop().expect("single climbing image");
        // fix forces for climbing image
        let freal = &forces[imax];
        let tangent = &tangents[imax - 1];
        forces_neb[imax] = freal - 2.0 * freal.dot(tangent) * tangent;

        Ok(())
    }

    // collect image energies and forces for later use
    fn collect_energies_and_forces(&self) -> Result<(Vec<f64>, Vec<Vector3fVec>)> {
        let n = self.images.len();
        let mut energies = Vec::with_capacity(n);
        let mut forces   = Vec::with_capacity(n);

        for i in 0..n {
            if let Some(e) = &self.images[i].energy {
                energies.push(*e);
            } else {
                bail!("image {}: no energy record.", i);
            }
            if let Some(f) = &self.images[i].forces {
                // FIXME: avoid allocation
                forces.push(f.clone());
            } else {
                bail!("image {}: no forces record.", i);
            }
        }

        Ok((energies, forces))
    }
}

// Calculate the parallel component of the spring force
// Parameters
// ----------
// displacements: displacement vectors between neighboring molecules (size = N - 1)
// spring_constants: spring constants for neighboring images (size = N - 1)
// tangents: tangents to NEB path for intermediate images excluding endpoints (size = N - 2)
fn spring_forces_parallel(displacements: &Vec<Vector3fVec>,
                          spring_constants: &Vec<f64>,
                          tangents: &Vec<Vector3fVec>) -> Vec<Vector3fVec>
{
    let nmols = tangents.len() + 2;
    debug_assert!(nmols - 1 == displacements.len());
    debug_assert!(nmols - 1 == spring_constants.len());

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
        // tangent vector corresponding to current image
        let tangent = &tangents[i-1];
        let f = (displ_next.norm() * knext - displ_prev.norm() * kprev) * tangent;
        forces.push(f);
    }
    forces
}

// Calculate the perpendicular component of the real forces
// Parameters
// ----------
// all_forces: real forces of molecule in each image including endpoints (size = N)
// tangents: tangent vectors of all intermediate images excluding endpoints (size = N - 2)
fn real_forces_perpendicular(all_forces: &Vec<Vector3fVec>, tangents: &Vec<Vector3fVec>) -> Vec<Vector3fVec> {
    let nmols = all_forces.len();
    debug_assert!(nmols - 2 == tangents.len());

    let mut vforces = Vec::with_capacity(nmols - 2);

    // loop over intermediate molecules excluding endpoints
    for i in 1..(nmols - 1) {
        let fi = &all_forces[i];
        // tangent vector corresponding to current image
        let ti = &tangents[i-1];
        let f = fi - fi.dot(ti) * ti;
        vforces.push(f);
    }

    vforces
}

// combine forces of all images into one
fn forces_mat_to_vec(arr_forces_mat: &[Vector3fVec]) -> Vec<Point3D> {
    let mut forces = vec![];
    for forces_mat in arr_forces_mat {
        let n = forces_mat.ncols();
        for i in 0..n {
            let v = forces_mat.column(i);
            let mut p = [0.0; 3];
            p[0] = v[0];
            p[1] = v[1];
            p[2] = v[2];
            forces.push(p);
        }
    }

    forces
}
// core:1 ends here

// utils

// [[file:~/Workspace/Programming/gosh/gosh.note::*utils][utils:1]]
// Return displacement vectors: positions_next - positions_this
fn get_displacements_between(positions_next: &Vec<Point3D>, positions_this: &Vec<Point3D>) -> Vector3fVec {
    debug_assert!(positions_this.len() == positions_next.len());

    let pts1 = positions_this.to_dmatrix();
    let pts2 = positions_next.to_dmatrix();

    pts2 - pts1
}

// Return displacement vectors between every pair of neighboring images
// displ = R_{i+1} - R_{i}
fn get_neighboring_images_displacements(images: &Vec<Image>) -> Result<Vec<Vector3fVec>>
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
// utils:1 ends here

// original

// [[file:~/Workspace/Programming/gosh/gosh.note::*original][original:1]]
// original algorithm for tangent calculation
// Ref: Classical and Quantum Dynamics in Condensed Phase Simulations; World Scientific, 1998; p 385.
fn tangent_vectors_original(displacements: &Vec<Vector3fVec>) -> Result<Vec<Vector3fVec>>
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
// original:1 ends here

// improved tangent

// [[file:~/Workspace/Programming/gosh/gosh.note::*improved%20tangent][improved tangent:1]]
// Parameters
// ----------
// displacements: displacement vectors between neighboring images
// energies: energies of molecules in images
//
// Reference
// ---------
// Henkelman, G.; Jónsson, H. J. Chem. Phys. 2000, 113 (22), 9978–9985.
//
fn tangent_vectors_improved
    (
        displacements: &Vec<Vector3fVec>,
        energies: &Vec<f64>
    ) -> Result<Vec<Vector3fVec>>
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
// improved tangent:1 ends here

// elastic band

// [[file:~/Workspace/Programming/gosh/gosh.note::*elastic%20band][elastic band:1]]
fn tangent_vectors_elastic_band(images: &Vec<Molecule>) -> Result<Vec<Vector3fVec>>
{
    unimplemented!()
}
// elastic band:1 ends here
