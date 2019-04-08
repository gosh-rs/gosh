// cgdescent.rs
// :PROPERTIES:
// :header-args: :tangle src/apps/optimization/cg_descent.rs
// :END:

// [[file:~/Workspace/Programming/gosh/gosh.note::*cgdescent.rs][cgdescent.rs:1]]
use cg_descent::CGDescent;

/// Optimize molecule using blackbox model
pub fn cgdescent_opt<T: ChemicalModel>(mol: &Molecule, model: &T) -> Result<ModelProperties> {
    let mut cgd = CGDescent::default();
    cgd.param.PrintLevel = 1;

    let mp = model.compute(&mol)?;
    if let Some(energy) = mp.energy {
        println!("current energy = {:-10.4}", energy);
    } else {
        bail!("no energy")
    }
    cgd.set_val_fn();
    cgd_set_grd_fn(|arr_x, gx| {
        mol.set_positions(&arr_x.as_positions());
        let mp = model.compute(&mol)?;

        // set gradients
        if let Some(forces) = mp.forces {
            let forces = forces.as_flat();
            assert_eq!(gx.len(), forces.len());
            for i in 0..forces.len() {
                gx[i] = - forces[i];
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
    });

    let mut mol = mol.clone();
    let mut positions = mol.positions();
    let mut arr_x = positions.as_mut_flat();
    cgd.run(&mut arr_x)?;

    let mp = model.compute(&mol)?;
    Ok(mp)
}
// cgdescent.rs:1 ends here
