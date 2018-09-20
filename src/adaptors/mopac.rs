// base

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
    fn parse_all(&self, output: &str) -> Result<Vec<ModelProperties>> {
        parse_mopac_output(&output)
    }
}

fn parse_mopac_output(output: &str) -> Result<Vec<ModelProperties>> {
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


    Ok(all_results)
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

// nom

use gchemol::parser::*;

/// whitespace including one or more spaces or tabs
named!(space_token<&str, &str>, eat_separator!(&b" \t"[..]));
macro_rules! sp (
    ($i:expr, $($args:tt)*) => (
        {
            sep!($i, space_token, $($args)*)
        }
    )
);

//  **                                MOPAC2016                                  **
named!(mopac_header<&str, (&str, &str)>, do_parse!(
       sp!(tag!("**"))    >>
    m: sp!(tag!("MOPAC")) >>
    v: digit              >>
       sp!(tag!("**"))    >>
    (
        (m, v)
    )
));

#[test]
fn test_mopac_header() {
    let line = " **                                MOPAC2016                                  **";
    let (r, (m, v)) = mopac_header(line).unwrap();
    assert!(r == "");
    assert!(m == "MOPAC");
    assert!(v == "2016");
}


//           TOTAL ENERGY            =       -720.18428 EV
named!(total_energy<&str, f64>, do_parse!(
            sp!(tag!("TOTAL ENERGY")) >>
            sp!(tag!("="))            >>
    energy: sp!(double_s)             >>
            sp!(tag!("EV"))           >>
    (energy)
));

#[test]
fn test_mopac_energy() {
    let line = "          TOTAL ENERGY            =       -720.18428 EV";
    let (r, en) = total_energy(line).unwrap();
    assert!(r == "");
    assert_relative_eq!(-720.18428, en, epsilon=1e-4);
}

//  DIPOLE           X         Y         Z       TOTAL
//  POINT-CHG.    -0.521    -0.058     0.081     0.531
//  HYBRID        -0.027    -0.069    -0.010     0.075
//  SUM           -0.548    -0.127     0.071     0.567
named!(dipole<&str, [f64; 3]>, do_parse!(
        sp!(tag!("DIPOLE")) >> read_until_eol >>
        read_until_eol      >>
        read_until_eol      >>
        sp!(tag!("SUM"))    >>
    x:  sp!(double_s)       >>
    y:  sp!(double_s)       >>
    z:  sp!(double_s)       >>
        read_until_eol      >>

    (
        [x, y, z]
    )
));

#[test]
fn test_mopac_dipole() {
    let txt = " DIPOLE           X         Y         Z       TOTAL
 POINT-CHG.    -0.521    -0.058     0.081     0.531
 HYBRID        -0.027    -0.069    -0.010     0.075
 SUM           -0.548    -0.127     0.071     0.567
";
    let (r, [x, y, z]) = dipole(txt).unwrap();
    assert_eq!(-0.548, x);
    assert_eq!(-0.127, y);
    assert_eq!(0.071, z);
}

//        FINAL  POINT  AND  DERIVATIVES
//
// PARAMETER     ATOM    TYPE            VALUE       GRADIENT
//     1          1  C    CARTESIAN X    -1.644300   -55.598091  KCAL/ANGSTROM
//     2          1  C    CARTESIAN Y    -0.817800    35.571574  KCAL/ANGSTROM
//     3          1  C    CARTESIAN Z     0.125500   -22.556128  KCAL/ANGSTROM
//     4          2  C    CARTESIAN X     1.631900     2.353930  KCAL/ANGSTROM
//     5          2  C    CARTESIAN Y    -0.872000    -7.974745  KCAL/ANGSTROM
//     6          2  C    CARTESIAN Z    -0.127300    12.066852  KCAL/ANGSTROM

named!(sym_position_gradient<&str, (&str, f64, f64)>, do_parse!(
              sp!(digit)                               >>
              sp!(digit)                               >>
    sym     : sp!(alpha)                               >>
              sp!(tag!("CARTESIAN"))                   >>
              sp!(alt!(tag!("X")|tag!("Y")|tag!("Z"))) >>
    position: sp!(double_s)                            >>
    gradient: sp!(double_s)                            >>
              read_until_eol                           >>
    (
        (sym, position, gradient)
    )
));

#[test]
fn test_sym_position_gradient() {
    let line = "      4          2  C    CARTESIAN X     1.523300     6.893093  KCAL/ANGSTROM\n";
    let (_, (sym, position, gradient)) = sym_position_gradient(line).unwrap();
    assert_eq!("C", sym);
    assert_eq!(1.523300, position);
    assert_eq!(6.893093, gradient);
}

named!(get_atom<&str, Atom>, do_parse!(
    px: sym_position_gradient >>
    py: sym_position_gradient >>
    pz: sym_position_gradient >>
    (
        Atom::build()
            .symbol(px.0)
            .position(px.1, py.1, pz.1)
            .momentum(px.2, py.2, pz.2)
            .finish()
    )
));

named!(get_atoms<&str, Vec<Atom>>, do_parse!(
           sp!(tag!("PARAMETER")) >> read_until_eol >>
    atoms: many1!(get_atom) >>

    (
        atoms
    )
));

#[test]
fn test_atoms() {
    let txt = "   PARAMETER     ATOM    TYPE            VALUE       GRADIENT
      1          1  C    CARTESIAN X    -1.743000   -80.695675  KCAL/ANGSTROM
      2          1  C    CARTESIAN Y    -0.725100    73.306387  KCAL/ANGSTROM
      3          1  C    CARTESIAN Z     0.044900   -23.565223  KCAL/ANGSTROM
      4          2  C    CARTESIAN X     1.523300     6.893093  KCAL/ANGSTROM
      5          2  C    CARTESIAN Y    -0.946300   -16.682683  KCAL/ANGSTROM
      6          2  C    CARTESIAN Z    -0.005100    22.532087  KCAL/ANGSTROM
      7          3  C    CARTESIAN X    -1.248600   -12.624765  KCAL/ANGSTROM
      8          3  C    CARTESIAN Y     0.589400   -35.843890  KCAL/ANGSTROM
      9          3  C    CARTESIAN Z    -0.026800     1.107735  KCAL/ANGSTROM
     10          4  C    CARTESIAN X     1.222600   -40.743520  KCAL/ANGSTROM
     11          4  C    CARTESIAN Y     0.386900    34.401001  KCAL/ANGSTROM
     12          4  C    CARTESIAN Z     0.076200    -5.837845  KCAL/ANGSTROM\n\n";

    let (_, atoms) = get_atoms(txt).unwrap();
    assert_eq!(4, atoms.len());
    assert_eq!(atoms[0].momentum(), [-80.695675, 73.306387, -23.565223]);
    assert_eq!(atoms[0].position(), [-1.7430, -0.7251, 0.0449]);
}

// test

#[test]
fn test_mopac_parse() {
    use gchemol::io;
    let fname = "tests/files/models/mopac/mopac.out";

    let m = MOPAC();
    let mr = m.parse_outfile(fname).unwrap();

    assert_relative_eq!(-720.18428, mr.energy.unwrap(), epsilon=1e-4);

    let dipole = mr.dipole.unwrap();
    assert_relative_eq!(0.10742828, dipole[0], epsilon=1e-4);

    let forces = mr.forces.unwrap();
    assert_eq!(13, forces.len());
    assert_relative_eq!(0.33629483, forces[0][0], epsilon=1e-4);

    let mol = mr.molecule.unwrap();
    assert_eq!(13, mol.natoms());
}
