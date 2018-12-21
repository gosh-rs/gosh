// base

// [[file:~/Workspace/Programming/gosh/gosh.note::*base][base:1]]
//! Implementation of the limited memory BFGS algorithm.
//! # References
//! - https://github.com/siesta-project/flos/blob/master/flos/optima/lbfgs.lua

use super::*;
// base:1 ends here

// core

// [[file:~/Workspace/Programming/gosh/gosh.note::*core][core:1]]
use ::lbfgs::lbfgs;

/// Optimize molecule using blackbox model
/// # Parameters
/// - mol: target molecule
/// - model: chemical model for properties evaluation
/// - fmax: the max force for convergence
pub fn lbfgs_opt<T: ChemicalModel>(
    mol: &Molecule,
    model: &T,
    fmax: f64,
) -> Result<ModelProperties> {
    let mp = model.compute(&mol)?;
    if let Some(energy) = mp.energy {
        println!("current energy = {:-10.4}", energy);
    } else {
        bail!("no energy")
    }

    let mut mol = mol.clone();
    let mut positions = mol.positions();
    let mut arr_x = positions.as_mut_flat();
    lbfgs().with_epsilon(fmax).minimize(
        &mut arr_x,
        |arr_x, gx| {
            mol.set_positions(&arr_x.as_positions());
            let mp = model.compute(&mol)?;

            // set gradients
            if let Some(forces) = mp.forces {
                let forces = forces.as_flat();
                assert_eq!(gx.len(), forces.len());
                for i in 0..forces.len() {
                    gx[i] = -forces[i];
                }
            } else {
                bail!("no forces!");
            }

            let fx = if let Some(energy) = mp.energy {
                energy
            } else {
                bail!("no energy!");
            };

            Ok(fx)
        },
        |prgr| {
            println!(
                "niter = {:}, fx = {:-10.4}, gnorm = {:-10.4}",
                prgr.niter, prgr.fx, prgr.gnorm
            );
            false
        },
    )?;

    let mp = model.compute(&mol)?;
    Ok(mp)
}
// core:1 ends here
