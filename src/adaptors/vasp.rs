// base

use super::*;
use std::fs::File;
use std::str::Lines;

use gchemol::{io, Atom, Molecule};

// unit conversion
const DEBYE: f64 = 0.20819434;
const KCAL_MOL: f64 = 1.0 / 23.061;

pub struct Vasp();

impl ModelAdaptor for Vasp {
    fn parse_all<P: AsRef<Path>>(&self, outfile: P) -> Result<Vec<ModelProperties>> {
        let outfile = outfile.as_ref();
        let f = File::open(outfile)?;

        let mut parser = TextParser::default();
        let mut all = vec![];
        parser.parse(f, vasp_outcar, |p| all.push(p))?;

        Ok(all)
    }

    fn parse_last<P: AsRef<Path>>(&self, outfile: P) -> Result<ModelProperties> {
        let outfile = outfile.as_ref();
        let f = File::open(outfile)?;

        let mut parser = TextParser::default();
        let mut last = ModelProperties::default();
        parser.parse(f, vasp_outcar, |p| last = p)?;

        Ok(last)
    }
}

// nom

use text_parser::*;

// force consistent energy
// free  energy   TOTEN  =      -536.930012 eV
named!(get_total_energy<&str, f64>, do_parse!(
            take_until!("free  energy   TOTEN  =") >>
            tag!("free  energy   TOTEN")           >>
            sp!(tag!("="))                         >>
    energy: sp!(double)                            >>
            sp!(tag!("eV"))                        >>
    (energy)
));

#[test]
fn test_vasp_energy() {
    let line = "  FREE ENERGIE OF THE ION-ELECTRON SYSTEM (eV)
  ---------------------------------------------------
  free  energy   TOTEN  =      -536.628381 eV

  energy  without entropy=     -536.775651  energy(sigma->0) =     -536.677471

";
    let (_, en) = get_total_energy(line).unwrap();
    // assert!(r == "");
    assert_relative_eq!(-536.628381, en, epsilon=1e-4);
}

// POSITION                                       TOTAL-FORCE (eV/Angst)
//     -----------------------------------------------------------------------------------
//     3.13915      4.47145      7.05899        -0.051556     -0.016880     -0.033586
//     5.48130      2.80880      7.05969         0.000184     -0.045212     -0.032849
//     5.83773      5.79740      6.84087        -0.577800     -0.751742     -0.808742
//     6.97326      7.41268      7.95898         0.339121      0.480393     -2.202270
//     1.15573      1.63445     -0.00000        -0.012611      0.019147     -0.003608
named!(total_force<&str, [f64; 3]>, do_parse!(
       sp!(double)                              >>
       sp!(double)                              >>
       sp!(double)                              >>
    x: sp!(double)                              >>
    y: sp!(double)                              >>
    z: sp!(double)                              >>
       read_until_eol                           >>
    ([x, y, z])
));

named!(get_vasp_forces<&str, Vec<[f64; 3]>>, do_parse!(
            take_until!("TOTAL-FORCE (eV/Angst)") >>
            read_until_eol                        >>
            read_until_eol                        >>
    forces: many1!(total_force)                   >>
    (forces)
));

#[test]
fn test_vasp_forces() {
    let line = " POSITION                                       TOTAL-FORCE (eV/Angst)
 -----------------------------------------------------------------------------------
      3.13915      4.47145      7.05899        -0.051556     -0.016880     -0.033586
      5.48130      2.80880      7.05969         0.000184     -0.045212     -0.032849
      5.83773      5.79740      6.84087        -0.577800     -0.751742     -0.808742
      6.97326      7.41268      7.95898         0.339121      0.480393     -2.202270
      1.15573      1.63445     -0.00000        -0.012611      0.019147     -0.003608
      1.15573      4.08614     -0.00000         0.007421      0.011830      0.002581
      1.15573      6.53782     -0.00000         0.002656     -0.026602      0.000487
      1.15573      8.98950     -0.00000         0.001754     -0.000284     -0.051374
      3.46720      2.45168     -0.00000         0.007068      0.007910     -0.002521
      3.46720      4.90336     -0.00000         0.010139      0.010535      0.040481
      3.46720      7.35505     -0.00000         0.039958     -0.044033      0.015392
 -----------------------------------------------------------------------------------
    total drift:                               -0.004104      0.000869     -0.010144
";
    let (_, forces) = get_vasp_forces(line).unwrap();
    assert_eq!(11, forces.len());
}

named!(vasp_outcar<&str, ModelProperties>, do_parse!(
    forces: get_vasp_forces                       >>
    energy: get_total_energy                      >>
    (
        {
            let mut p = ModelProperties::default();
            p.energy = Some(energy);
            p.forces = Some(forces);

            p
        }
    )
));
