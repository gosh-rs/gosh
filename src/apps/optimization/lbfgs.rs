// base

// [[file:~/Workspace/Programming/gosh-rs/gosh/gosh.note::*base][base:1]]
//! Implementation of the limited memory BFGS algorithm.
//! # References
//! - https://github.com/siesta-project/flos/blob/master/flos/optima/lbfgs.lua

use super::*;
// base:1 ends here

// core

// [[file:~/Workspace/Programming/gosh-rs/gosh/gosh.note::*core][core:1]]
use ::lbfgs::lbfgs;

use gosh_db::prelude::*;

/// Optimize molecule using blackbox model
/// # Parameters
/// - mol: target molecule
/// - model: chemical model for properties evaluation
/// - fmax: the max force for convergence
pub fn lbfgs_opt<T: ChemicalModel>(
    mol: &Molecule,
    model: &mut T,
    fmax: f64,
) -> Result<ModelProperties> {
    let mp = model.compute(&mol)?;
    if let Some(energy) = mp.energy {
        println!("current energy = {:-10.4}", energy);
    } else {
        bail!("no energy")
    }

    // use db for checkpointing opt data
    let ckpt_conn = gosh_db::DbConnection::establish();

    let mut mol = mol.clone();
    let mut positions = mol.positions();
    let mut arr_x = positions.as_mut_flat();
    let mut icall = 0;
    lbfgs()
        .with_initial_step_size(1.0 / 75.0)
        .with_max_step_size(0.4)
        .with_damping(true)
        .with_max_linesearch(5)
        .with_linesearch_gtol(0.999)
        .with_epsilon(0.01)
        .minimize(
            &mut arr_x,
            |arr_x, gx| {
                mol.set_positions(&arr_x.as_positions());
                let mp = model.compute(&mol)?;

                // set gradients
                if let Some(forces) = &mp.forces {
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

                let fnorms: Vec<_> = gx.as_positions().iter().map(|v| v.norm()).collect();
                let fm = fnorms.max();
                println!(
                    "niter = {:5} energy = {:-10.4} fmax = {:-10.4}",
                    icall, fx, fm
                );
                icall += 1;

                // checkpointing
                if let Ok(conn) = &ckpt_conn {
                    mp.checkpoint(&conn)?;
                }

                Ok(fx)
            },
            |prgr| {
                let fnorms: Vec<_> = prgr.gx.chunks(3).map(|v| v.norm()).collect();
                let fcur = fnorms.max();
                //let fcur = crate::vecfx::max(fnorms);
                fcur <= fmax
            },
        )?;

    let mp = model.compute(&mol)?;
    Ok(mp)
}
// core:1 ends here
