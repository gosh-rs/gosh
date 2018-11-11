// base

use super::*;
use std::fs::File;
use std::str::Lines;

use gchemol::{
    io,
    Atom,
    Molecule,
};

// unit conversion
const DEBYE: f64 = 0.20819434;
const KCAL_MOL: f64 = 1.0/23.061;

pub struct MOPAC();

impl ModelAdaptor for MOPAC {
    fn parse_all<P: AsRef<Path>>(&self, outfile: P) -> Result<Vec<ModelProperties>> {
        let outfile = outfile.as_ref();
        let f = File::open(outfile)?;

        let mut parser = TextParser::default();
        let mut all = vec![];
        parser.parse(f, mopac_output, |p| all.push(p))?;

        Ok(all)
    }

    fn parse_last<P: AsRef<Path>>(&self, outfile: P) -> Result<ModelProperties> {
        let outfile = outfile.as_ref();
        let f = File::open(outfile)?;

        let mut parser = TextParser::default();
        let mut last = ModelProperties::default();
        parser.parse(f, mopac_output, |p| last = p)?;

        Ok(last)
    }
}

// nom

use textparser::*;

// /// A whitespace wrapper consuming " \t\r" (no newline)
// named!(pub space_token<&str, &str>, eat_separator!(&b" \t\r"[..]));

// #[macro_export]
// macro_rules! sp (
//     ($i:expr, $($args:tt)*) => (
//         {
//             sep!($i, space_token, $($args)*)
//         }
//     )
// );

//           TOTAL ENERGY            =       -720.18428 EV
named!(get_total_energy<&str, f64>, do_parse!(
            take_until!("TOTAL ENERGY            =") >>
            tag!("TOTAL ENERGY")                     >>
            sp!(tag!("="))                           >>
    energy: sp!(double)                              >>
            sp!(tag!("EV"))                          >>
    (energy)
));

#[test]
fn test_mopac_energy() {
    let line = "TOTAL ENERGY            =       -720.18428 EV";
    let (r, en) = get_total_energy(line).unwrap();
    assert!(r == "");
    assert_relative_eq!(-720.18428, en, epsilon=1e-4);
}

//  DIPOLE           X         Y         Z       TOTAL
//  POINT-CHG.    -0.521    -0.058     0.081     0.531
//  HYBRID        -0.027    -0.069    -0.010     0.075
//  SUM           -0.548    -0.127     0.071     0.567
named!(get_dipole<&str, [f64; 3]>, do_parse!(
        take_until!("DIPOLE           X         Y         Z") >> read_until_eol >>
        read_until_eol      >>
        read_until_eol      >>
        sp!(tag!("SUM"))    >>
    x:  sp!(double)         >>
    y:  sp!(double)         >>
    z:  sp!(double)         >>

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
    let (r, [x, y, z]) = get_dipole(txt).unwrap();
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
    position: sp!(double)                              >>
    gradient: sp!(double)                              >>
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
    take_until!("FINAL  POINT  AND  DERIVATIVES") >> read_until_eol >>
           read_until_eol                         >>
           read_until_eol                         >>
    atoms: many1!(get_atom)                       >>

    (
        atoms
    )
));

#[test]
fn test_atoms() {
    let txt = "       FINAL  POINT  AND  DERIVATIVES

   PARAMETER     ATOM    TYPE            VALUE       GRADIENT
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

named!(mopac_output<&str, ModelProperties>, do_parse!(
    // make sure it is a normal mopac output file
    take_until!("Cite this program as: MOPAC") >>
    // force consistent energy
    energy : get_total_energy                  >>
    // structure and gradients (stored as momentum)
    atoms  : opt!(complete!(get_atoms))        >>
    // dipole moment
    dipole : get_dipole                        >>

    (
        {
            let mut p = ModelProperties::default();
            p.energy = Some(energy);
            p.dipole = Some([
                dipole[0] * DEBYE,
                dipole[1] * DEBYE,
                dipole[2] * DEBYE,
            ]);

            let mut mol = Molecule::new("mopac");
            let mut forces = vec![];
            if let Some(atoms) = atoms {
                for a in atoms {
                    let [x, y, z] = a.momentum();

                    forces.push([
                        -x * KCAL_MOL,
                        -y * KCAL_MOL,
                        -z * KCAL_MOL,
                    ]);
                    mol.add_atom(a);
                }
                p.forces = Some(forces);
                p.molecule = Some(mol);
            }

            p
        }
    )
));

// test

#[test]
fn test_mopac_parse() {
    use gchemol::io;
    let fname = "tests/files/models/mopac/mopac-grad-1scf.out";

    let m = MOPAC();
    let mr = m.parse_last(fname).unwrap();

    assert_relative_eq!(-720.18428, mr.energy.unwrap(), epsilon=1e-4);

    let dipole = mr.dipole.unwrap();
    assert_relative_eq!(0.10742828, dipole[0], epsilon=1e-4);

    let forces = mr.forces.unwrap();
    assert_eq!(13, forces.len());
    assert_relative_eq!(0.33629483, forces[0][0], epsilon=1e-4);

    let mol = mr.molecule.unwrap();
    assert_eq!(13, mol.natoms());

    // parsing single point energy calculations
    let fname = "tests/files/models/mopac/mopac-sp.out";

    let m = MOPAC();
    let mr = m.parse_last(fname).unwrap();
    assert_eq!(Some(-748.27010), mr.energy)
}
