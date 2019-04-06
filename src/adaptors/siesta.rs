// base

use super::*;
use std::fs::File;
use std::str::Lines;

use gchemol::{io, Atom, Molecule};

// unit conversion
const DEBYE: f64 = 0.20819434;
const KCAL_MOL: f64 = 1.0 / 23.061;

pub struct Siesta();

impl ModelAdaptor for Siesta {
    fn parse_all<P: AsRef<Path>>(&self, outfile: P) -> Result<Vec<ModelProperties>> {
        let outfile = outfile.as_ref();
        let f = File::open(outfile)?;

        let mut parser = TextParser::default();
        let mut all = vec![];
        parser.parse(f, siesta_output, |p| all.push(p))?;

        Ok(all)
    }

    fn parse_last<P: AsRef<Path>>(&self, outfile: P) -> Result<ModelProperties> {
        let outfile = outfile.as_ref();
        let f = File::open(outfile)?;

        let mut parser = TextParser::default();
        let mut last = ModelProperties::default();
        parser.parse(f, siesta_output, |p| last = p)?;

        Ok(last)
    }
}

// nom

use text_parser::*;

// force consistent energy
// siesta:         Total =  -37729.793337
named!(get_total_energy<&str, f64>, do_parse!(
            take_until!("siesta:         Total =") >>
            tag!("siesta:         Total =")        >>
    energy: sp!(double)                            >>
            read_until_eol                         >>
    (energy)
));

#[test]
fn test_energy() {
    let line = "
siesta: Final energy (eV):
siesta:  Band Struct. =   -8565.028584
siesta:       Kinetic =   24443.230452
siesta:       Hartree =   30079.877056
siesta:       Eldau   =       0.000000
siesta:       Eso     =       0.000000
siesta:    Ext. field =       0.000000
siesta:       Enegf   =       0.000000
siesta:   Exch.-corr. =  -10410.095652
siesta:  Ion-electron =  -81937.616966
siesta:       Ion-ion =      94.811773
siesta:       Ekinion =       0.000000
siesta:         Total =  -37729.793337
siesta:         Fermi =      -2.515979
siesta:         Total =  -37729.793337\n
";
    let (_, en) = get_total_energy(line).unwrap();
    assert_relative_eq!(-37729.793337, en, epsilon=1e-4);
}

// siesta: Atomic forces (eV/Ang):
// siesta:      1   -0.150976   -0.222301   -0.235821
// siesta:      2    0.116304    0.010505   -0.263769
// siesta:      3    0.247099    0.686634   -0.145696
// siesta:      4   -0.135647   -0.223600    0.050641
// siesta:      5   -0.189407    0.281690    0.144560
named!(total_force<&str, [f64; 3]>, do_parse!(
       tag!("siesta:")     >>
       sp!(unsigned_digit) >>
    x: sp!(double)         >>
    y: sp!(double)         >>
    z: sp!(double)         >>
       read_until_eol      >>
    ([x, y, z])
));

named!(get_forces<&str, Vec<[f64; 3]>>, do_parse!(
            take_until!("siesta: Atomic forces (eV/Ang):") >>
            read_until_eol                                 >>
    forces: many1!(total_force)                            >>
    (forces)
));

#[test]
fn test_forces() {
    let line = "
siesta: Atomic forces (eV/Ang):
siesta:      1   -0.150976   -0.222301   -0.235821
siesta:      2    0.116304    0.010505   -0.263769
siesta:      3    0.247099    0.686634   -0.145696
siesta:      4   -0.135647   -0.223600    0.050641
siesta:      5   -0.189407    0.281690    0.144560
siesta:      6    0.279415    0.026510    0.249805
siesta:      7   -0.210915   -0.360380   -0.190146
siesta:      8    0.002322   -0.165403   -0.691606
siesta:      9   -0.238029    0.144006   -0.154580
siesta:     10   -0.231124   -0.283278    0.247035
siesta:     11   -0.409292   -0.090288    0.810874
siesta:     12    0.174531   -0.124098    0.324220
siesta:     13   -0.102700   -0.275183   -0.289818
siesta:     14    0.355493   -0.030670   -0.308250
siesta: ----------------------------------------
siesta:    Tot   -0.352038    0.344321   -0.600608
";
    let (_, forces) = get_forces(line).unwrap();
    assert_eq!(14, forces.len());
}

named!(siesta_output<&str, ModelProperties>, do_parse!(
    energy: get_total_energy                      >>
    forces: get_forces                            >>
    (
        {
            let mut p = ModelProperties::default();
            p.energy = Some(energy);
            p.forces = Some(forces);

            p
        }
    )
));
