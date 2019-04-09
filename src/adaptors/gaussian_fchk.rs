// base

use super::*;
use std::fs::File;
use std::str::Lines;

use gchemol::{io, Atom, Molecule};

// unit conversion
const DEBYE: f64 = 0.20819434;
const KCAL_MOL: f64 = 1.0 / 23.061;
// Hartree to eV
const HARTREE: f64 = 27.211386024367243;
const BOHR: f64 = 0.5291772105638411;

pub struct GaussianFchk();

impl ModelAdaptor for GaussianFchk {
    fn parse_all<P: AsRef<Path>>(&self, outfile: P) -> Result<Vec<ModelProperties>> {
        let outfile = outfile.as_ref();
        let f = File::open(outfile)?;

        let mut parser = TextParser::default();
        let mut all = vec![];
        parser.parse(f, gaussian_fchk, |p| all.push(p))?;

        Ok(all)
    }

    fn parse_last<P: AsRef<Path>>(&self, outfile: P) -> Result<ModelProperties> {
        let outfile = outfile.as_ref();
        let f = File::open(outfile)?;

        let mut parser = TextParser::default();
        let mut last = ModelProperties::default();
        parser.parse(f, gaussian_fchk, |p| last = p)?;

        Ok(last)
    }
}

// nom

use text_parser::*;

// Total Energy                               R     -4.019490631335482E+01
named!(get_total_energy<&str, f64>, do_parse!(
            take_until!("Total Energy                               R") >>
            tag!("Total Energy                               R")        >>
    energy: sp!(double) >> eol >>
    (energy * HARTREE)
));

#[test]
fn test_energy() {
    let line = "
Number of contracted shells                I               12
Highest angular momentum                   I                2
Largest degree of contraction              I                6
Number of primitive shells                 I               27
Viriral Ratio                              R      2.001677529178424E+00
SCF Energy                                 R     -4.019490631335482E+01
Total Energy                               R     -4.019490631335482E+01
RMS Density                                R      6.140789652789450E-05
";
    let (_, en) = get_total_energy(line).unwrap();
    assert_relative_eq!(-40.194906*HARTREE, en, epsilon = 1e-4);
}

named!(force_record<&str, [f64; 3]>, do_parse!(
    x: ws!(double)         >>
    y: ws!(double)         >>
    z: ws!(double)         >>
    (
        // gradient to force
        [
            -1.0 * x * HARTREE / BOHR,
            -1.0 * y * HARTREE / BOHR,
            -1.0 * z * HARTREE / BOHR
        ]
    )
));

named!(get_forces<&str, Vec<[f64; 3]>>, do_parse!(
            take_until!("Cartesian Gradient")     >>
            read_until_eol                        >>
    forces: many1!(force_record)                  >>
    (forces)
));

#[test]
fn test_forces() {
    let line = "
Coordinates of each shell                  R   N=          33
 -6.57759708E+00  3.18941117E-01  8.61053146E-02 -5.90361788E+00 -1.58743350E+00
  8.61053146E-02 -5.90358309E+00  1.27211556E+00 -1.56485678E+00 -8.59960404E+00
  3.18966024E-01  8.61053146E-02 -5.60752087E+00  1.69079965E+00  2.46225635E+00
 -6.28455137E+00  7.39760148E-01  4.11321538E+00 -3.58551733E+00  1.68757449E+00
  2.46410592E+00 -6.57320056E+00  4.43608527E+00  2.45959769E+00 -8.59520332E+00
  4.43930947E+00  2.45703723E+00 -5.89976736E+00  5.38905576E+00  4.11091448E+00
 -5.89558993E+00  5.38732851E+00  8.08994082E-01
Cartesian Gradient                         R   N=          33
 -8.28320853E-03 -3.02626864E-03 -1.53180257E-02 -6.75259487E-03  2.43760591E-02
  3.03611414E-03 -6.93471874E-03 -7.65136760E-03  2.39128846E-02  2.54421523E-02
  3.77588317E-03  4.07939033E-03  1.49673816E-02 -1.06062356E-02  1.84041695E-02
  5.79468768E-03  1.36277343E-02 -2.36507676E-02 -2.76623363E-02  1.88144286E-03
 -3.21440685E-03 -8.25647235E-03  1.47569137E-02 -5.05420455E-03  2.54396064E-02
 -5.47193885E-03 -1.19976921E-03 -6.76373840E-03 -1.48157825E-02 -1.95788197E-02
 -6.99075911E-03 -1.68464397E-02  1.85834353E-02
Cartesian Force Constants                  R   N=         561
  7.97086973E-01 -3.76404785E-02  7.77285013E-01 -6.33672974E-02 -8.77696466E-02
  6.68508852E-01 -1.05697313E-01  1.30432275E-01  1.92479175E-03  1.00635272E-01
";
    let (_, forces) = get_forces(line).unwrap();
    assert_eq!(11, forces.len());
}

named!(gaussian_fchk<&str, ModelProperties>, do_parse!(
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

// test

#[test]
fn test_gaussian_fchk() -> Result<()> {
    let fchkfile = "tests/files/C3H8/Test.FChk";

    let app = GaussianFchk();
    let mp = app.parse_last(fchkfile)?;
    assert!(mp.energy.is_some());
    assert_relative_eq!(-3203.5062872309163, mp.energy.unwrap(), epsilon = 1e-4);

    assert!(mp.forces.is_some());
    assert_relative_eq!(0.48565407, mp.forces.unwrap()[0][0], epsilon = 1e-4);

    Ok(())
}
