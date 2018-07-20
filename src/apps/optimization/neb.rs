// [[file:~/Workspace/Programming/gosh/gosh.note::b805a230-8b97-4b2e-ac45-0430598e1af8][b805a230-8b97-4b2e-ac45-0430598e1af8]]
/// Implementation of the Nudged Elastic Band (NEB) method for finding minimum energy paths and saddle points
///
/// References
/// ----------
/// (1) Henkelman & Jonsson, JCP (113), 2000
/// (2) Henkelman, Uberuaga, & Jonsson, JCP (113), 2000

use super::*;
use gchemol::Molecule;

pub struct NEB {
    /// NEB images: A list of Molecules
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
    //
}

fn tangent_vectors_original(images: &Vec<Molecule>) -> Vec<[f64; 3]> {
    unimplemented!()
}

// Henkelman, G.; Jónsson, H. J. Chem. Phys. 2000, 113 (22), 9978–9985.
fn tangent_vectors_improved(images: &Vec<Molecule>) -> Vec<[f64; 3]> {
    unimplemented!()
}

fn tangent_vectors_elastic_band(images: &Vec<Molecule>) -> Vec<[f64; 3]> {
    unimplemented!()
}
// b805a230-8b97-4b2e-ac45-0430598e1af8 ends here
