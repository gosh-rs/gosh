// base

use super::*;
use std::fs::File;
use std::str::Lines;

use gchemol::{io, Atom, Molecule};

// unit conversion
const DEBYE: f64 = 0.20819434;
const KCAL_MOL: f64 = 1.0 / 23.061;

pub struct Gulp();

impl ModelAdaptor for Gulp {
    fn parse_all<P: AsRef<Path>>(&self, outfile: P) -> Result<Vec<ModelProperties>> {
        let outfile = outfile.as_ref();
        let f = File::open(outfile)?;

        let mut parser = TextParser::default();
        let mut all = vec![];
        parser.parse(f, gulp_output, |p| all.push(p))?;

        Ok(all)
    }

    fn parse_last<P: AsRef<Path>>(&self, outfile: P) -> Result<ModelProperties> {
        let outfile = outfile.as_ref();
        let f = File::open(outfile)?;

        let mut parser = TextParser::default();
        let mut last = ModelProperties::default();
        parser.parse(f, gulp_output, |p| last = p)?;

        Ok(last)
    }
}

impl Gulp {
    // FIXME: adhoc hacking
    pub fn parse_piped(&self) -> Result<ModelProperties> {
        let f = std::io::stdin();
        let mut parser = TextParser::default();
        let mut last = ModelProperties::default();
        parser.parse(f, gulp_output, |p| last = p)?;

        Ok(last)
    }
}

// nom

use text_parser::*;

  // Final energy =     -91.56438967 eV
  // Final Gnorm  =       0.00027018
  named!(get_total_energy<&str, f64>, do_parse!(
              take_until!("Final energy =") >>
              tag!("Final energy =")        >>
      energy: sp!(double)                            >>
              read_until_eol                         >>
      (energy)
  ));

  #[test]
  fn test_energy() {
      let line = "
    **** Optimisation achieved ****


    Final energy =     -91.56438967 eV
    Final Gnorm  =       0.00027018

    Components of energy :

  --------------------------------------------------------------------------------
    Interatomic potentials     =           0.00000000 eV
    Bond-order potentials      =         -91.56438967 eV
    Monopole - monopole (real) =           0.00000000 eV
  --------------------------------------------------------------------------------
    Total lattice energy       =         -91.56438967 eV
  --------------------------------------------------------------------------------
    Total lattice energy       =           -8834.5639 kJ/mol
  --------------------------------------------------------------------------------
  ";
      let (_, en) = get_total_energy(line).unwrap();
      assert_relative_eq!(-91.56438967, en, epsilon=1e-4);
  }

  // --------------------------------------------------------------------------------
  //     1  C     c    -1.955930   -1.191761    0.755951    0.000000
  //     2  C     c    -0.500103   -1.240187   -1.361902    0.000000
  //     3  C     c    -0.102169    1.769707   -0.706440    0.000000
  //     4  C     c    -1.518660    1.211979    1.364784    0.000000
  named!(symbol_and_xyz<&str, (&str, [f64; 3])>, do_parse!(
         sp!(unsigned_digit) >>
      s: sp!(alpha)          >>
         sp!(alpha)          >>
      x: sp!(double)         >>
      y: sp!(double)         >>
      z: sp!(double)         >>
         read_until_eol      >>
      ((s, [x, y, z]))
  ));

  named!(get_structure<&str, Vec<(&str, [f64; 3])>>, do_parse!(
               take_until!("Final cartesian coordinates of atoms") >> read_until_eol >>
               read_until_eol                                 >>
               read_until_eol                                 >>
               read_until_eol                                 >>
               read_until_eol                                 >>
               read_until_eol                                 >>
      results: many1!(symbol_and_xyz)                         >>
      (results)
  ));

  // --------------------------------------------------------------------------------
  //     1  C     c    -1.955930   -1.191761    0.755951    0.000000
  //     2  C     c    -0.500103   -1.240187   -1.361902    0.000000
  //     3  C     c    -0.102169    1.769707   -0.706440    0.000000
  //     4  C     c    -1.518660    1.211979    1.364784    0.000000
  named!(total_force<&str, [f64; 3]>, do_parse!(
         sp!(unsigned_digit) >>
         sp!(alpha)          >>
         sp!(alpha)          >>
      x: sp!(double)         >>
      y: sp!(double)         >>
      z: sp!(double)         >>
         read_until_eol      >>
      ([x, y, z])
  ));

  named!(get_forces<&str, Vec<[f64; 3]>>, do_parse!(
              take_until!("Final Cartesian derivati") >> read_until_eol >>
              read_until_eol                                 >>
              read_until_eol                                 >>
              read_until_eol                                 >>
              read_until_eol                                 >>
              read_until_eol                                 >>
      forces: many1!(total_force)                            >>
      (forces)
  ));

  #[test]
  fn test_forces() {
      let line = "

  Final Cartesian derivatives :

--------------------------------------------------------------------------------
   No.  Atomic          x             y             z           Radius
        Label       (eV/Angs)     (eV/Angs)    (eV/Angs)      (eV/Angs)
--------------------------------------------------------------------------------
      1 C     c       0.002751      0.001390     -0.003469      0.000000
      2 C     c       0.001449      0.002452      0.003151      0.000000
      3 C     c       0.000000      0.000000      0.000000      0.000000
      4 C     c       0.000770     -0.000683     -0.002221      0.000000
      5 C     c      -0.001782      0.000383     -0.001570      0.000000
      6 C     c      -0.000510     -0.000158     -0.001059      0.000000
      7 C     c      -0.002169     -0.000789     -0.001802      0.000000
      8 C     c      -0.000586     -0.000156     -0.000930      0.000000
      9 C     c      -0.001191     -0.000769     -0.000263      0.000000
     10 C     c       0.000717      0.002025      0.002112      0.000000
     11 C     c       0.000607     -0.001269      0.000072      0.000000
     12 C     c       0.002147      0.000618     -0.002932      0.000000
     13 C     c      -0.003072     -0.000312     -0.000294      0.000000
     14 C     c      -0.000142     -0.003763      0.002650      0.000000
     15 C     c      -0.000068      0.002719      0.001997      0.000000
     16 C     c       0.001216      0.002239      0.003134      0.000000
--------------------------------------------------------------------------------
  Maximum abs         0.003072      0.003763      0.003469      0.000000
--------------------------------------------------------------------------------
  ";
      let (_, forces) = get_forces(line).unwrap();
      assert_eq!(16, forces.len());
  }

  named!(gulp_output<&str, ModelProperties>, do_parse!(
      energy: get_total_energy                      >>
      structure: get_structure                      >>
      forces: get_forces                            >>
      (
          {
              let mut p = ModelProperties::default();
              p.energy = Some(energy);
              p.forces = Some(forces);
              // construct molecule
              let mut mol = Molecule::new("gulp");
              for (sym, position) in structure {
                  let atom = Atom::new(sym, position);
                  mol.add_atom(atom);
              }
              p.molecule = Some(mol);

              p
          }
      )
  ));
