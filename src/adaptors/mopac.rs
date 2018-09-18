// mopac.rs
// :PROPERTIES:
// :header-args: :comments org :tangle src/adaptors/mopac.rs
// :END:

use super::*;
use std::str::Lines;
use std::ops::{Generator, GeneratorState};

use gchemol::{
    io,
    Atom,
    Molecule,
};

const DEBYE: f64 = 0.20819434;

pub struct MOPAC();

impl ModelAdaptor for MOPAC {
    fn parse_stream(&self, output: &str) -> Result<ModelProperties> {
        parse_mopac_output(output)
    }
}

fn parse_mopac_output(output: &str) -> Result<ModelProperties> {
    let mut lines = output.lines();

    let mut generator = || {
        let mut mresults = ModelProperties::default();

        let mut found = false;

        // collect energy
        if let Some(line) = get_tagged_line(&mut lines, "TOTAL ENERGY") {
            let parts: Vec<_> = line.split_whitespace().collect();
            assert_eq!(5, parts.len(), "expect line containing energy: {:?}", parts);
            let energy = parts[3].parse()?;
            mresults.energy = Some(energy);
            found = true;
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

        if let Some(_) = get_tagged_line(&mut lines, "FINAL  POINT  AND  DERIVATIVES") {
            // ignore the next two lines
            lines.next();
            lines.next();

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
                // kcal/mol to eV
                force[i] = grad / -23.061;
                position[i] = pos;
                if i == 2 {
                    forces.push(force.clone());
                    mol.add_atom(Atom::new(sym, position));
                }
            }

            if mol.natoms() < 1 {
                bail!("forces are incomplete!");
            }

            mresults.molecule = Some(mol);
            mresults.forces = Some(forces);
            found = true;
        }

        // dipole
        if let Some(_) = get_tagged_line(&mut lines, "DIPOLE") {
            lines.next();
            lines.next();

            if let Some(line) = lines.next() {
                let parts: Vec<_> = line.split_whitespace().collect();
                debug_assert_eq!(5, parts.len(), "{:?}", parts);
                let x: f64 = parts[1].parse()?;
                let y: f64 = parts[2].parse()?;
                let z: f64 = parts[3].parse()?;
                mresults.dipole = Some([x * DEBYE, y * DEBYE, z * DEBYE]);
                found = true;
            } else {
                warn!("incomplete file, missing dipole record.");
            }
        }

        if found {
            yield mresults;
        }

        Ok(())
    };

    let mut all_results = vec![];
    loop {
        match unsafe {generator.resume()} {
            GeneratorState::Yielded(mresults) => {
                all_results.push(mresults);
            },
            GeneratorState::Complete(_) => {
                break;
            }
        }
    }

    let n = all_results.len();
    if n != 0 {
        Ok(all_results[n-1].clone())
    } else {
        bail!("got nothing!");
    }

}

// jump to the line containg a special tag (using starts_with, ignoring leading
// spaces)
// Return the consumed line if found
fn get_tagged_line(lines: &mut Lines, tag: &str) -> Option<String> {
    for line in lines.by_ref() {
        if line.trim_left().starts_with(tag) {
            return Some(line.to_string())
        }
    }

    // EOF
    None
}
