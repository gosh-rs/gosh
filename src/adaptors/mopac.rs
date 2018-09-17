// mopac.rs
// :PROPERTIES:
// :header-args: :comments org :tangle src/adaptors/mopac.rs
// :END:

use quicli::prelude::*;
use gchemol::{
    io,
    Atom,
    Molecule,
};
use models::*;

pub fn parse_mopac_output(output: &str) -> Result<ModelResults> {
    let mut mresults = ModelResults::default();
    let mut lines = output.lines();

    // collect energy
    for line in lines.by_ref() {
        if line.trim_left().starts_with("TOTAL ENERGY") {
            let parts: Vec<_> = line.split_whitespace().collect();
            assert_eq!(5, parts.len(), "expect line containing energy: {:?}", parts);
            let energy = parts[3].parse()?;
            mresults.energy = Some(energy);
            break;
        }
    }

    // collect forces
    //
    // reference text:
    //        FINAL  POINT  AND  DERIVATIVES
    //
    // PARAMETER     ATOM    TYPE            VALUE       GRADIENT
    //     1          1  C    CARTESIAN X    -1.611300    -2.127778  KCAL/ANGSTROM
    //     2          1  C    CARTESIAN Y    -0.850600   -13.758186  KCAL/ANGSTROM
    //     3          1  C    CARTESIAN Z     0.144200     5.137547  KCAL/ANGSTROM
    // ...
    let mut found = false;
    for line in lines.by_ref() {
        if line.trim_left().starts_with("FINAL  POINT  AND  DERIVATIVES") {
            found = true;
            break;
        }
    }

    if found {
        // ignore two lines
        for _ in 0..2 {
            lines.next();
        }

        let mut mol = Molecule::new("mopc");
        let mut forces    = vec![];

        let mut force     = [0.0; 3];
        let mut position  = [0.0; 3];
        for (i, line) in lines.by_ref().enumerate() {
            let line = line.trim();

            // record ending
            if line.is_empty() {
                break;
            }

            let parts: Vec<_> = line.split_whitespace().collect();
            assert_eq!(8, parts.len(), "incorrect: {:?}", parts);
            let sym  = parts[2];
            let pos: f64  = parts[5].parse()?;
            let grad: f64 = parts[6].parse()?;
            let i = i % 3;
            force[i] = - grad;
            position[i] = pos;
            if i == 2 {
                forces.push(force.clone());
                mol.add_atom(Atom::new(sym, position));
            }
        }
        debug_assert_eq!(mol.natoms(), forces.len());
        mresults.molecule = Some(mol);
        mresults.forces = Some(forces);
    }

    // dipole
    let mut found = false;
    for line in lines.by_ref() {
        if line.trim_left().starts_with("DIPOLE") {
            found = true;
            break;
        }
    }
    if found {
        for _ in 0..2 {
            lines.next();
        }
        if let Some(line) = lines.next() {
            let parts: Vec<_> = line.split_whitespace().collect();
            debug_assert_eq!(5, parts.len(), "{:?}", parts);
            let x = parts[1].parse()?;
            let y = parts[2].parse()?;
            let z = parts[3].parse()?;
            mresults.dipole = Some([x, y, z]);
        } else {
            warn!("incomplete file, missing dipole record.");
        }
    }

    Ok(mresults)
}
