// [[file:~/Workspace/Programming/gosh/gosh.note::894d0c1b-0482-46b9-a3dc-8f00b78833bc][894d0c1b-0482-46b9-a3dc-8f00b78833bc]]
use quicli::prelude::*;
use std::path::{Path, PathBuf};

use gchemol::{
    io,
    Molecule
};

pub mod dftb;
pub mod lj;

#[derive(Debug, Clone)]
pub struct ModelResults {
    pub molecule        : Option<Molecule>,
    pub energy          : Option<f64>,
    pub forces          : Option<Vec<[f64; 3]>>,
    pub dipole_moment   : Option<[f64; 3]>,
    pub force_constants : Option<Vec<[f64; 3]>>,
}

impl Default for ModelResults {
    fn default() -> Self {
        ModelResults {
            molecule        : None,
            energy          : None,
            forces          : None,
            dipole_moment   : None,
            force_constants : None,
        }
    }
}

pub trait ChemicalModel {
    fn positions(&self) -> Vec<[f64; 3]> {
        unimplemented!()
    }

    /// define how to calculate properties, such as energy, forces, ...
    fn calculate(&self, mol: &Molecule) -> Result<ModelResults> {
        unimplemented!()
    }

    fn energy(&self) -> f64 {
        unimplemented!()
    }

    fn forces(&self) -> Vec<[f64; 3]> {
        unimplemented!()
    }

    fn dipole_moment(&self) -> [f64; 3] {
        unimplemented!()
    }

    // fn polarizability(&self) ->
    // fn dipole_derivatives(&self) ->
    // fn force_constants(&self) ->
}
// 894d0c1b-0482-46b9-a3dc-8f00b78833bc ends here

// [[file:~/Workspace/Programming/gosh/gosh.note::ea1864bb-6cc4-42f1-93f5-cebd790c58ab][ea1864bb-6cc4-42f1-93f5-cebd790c58ab]]
use std::str::FromStr;

impl FromStr for ModelResults {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        parse_model_results(s)
    }
}

fn parse_model_results(stream: &str) -> Result<ModelResults>{
    let mut results = ModelResults::default();

    let mut lines = stream.lines().peekable();
    loop {
        if let Some(line) = lines.next() {
            if line.starts_with("@") {
                let header = line.split_whitespace().next();
                match header {
                    Some("@total_energy") => {
                        if let Some(line) = lines.next() {
                            let energy = line
                                .trim()
                                .parse()?;
                            results.energy = Some(energy);
                        } else {
                            warn!("expected energy record not found!");
                            break;
                        }
                    },
                    Some("@forces") => {
                        // unimplemented!()
                    },
                    Some("@dipole_moments") => {
                        // unimplemented!()
                    },
                    Some("@structure") => {
                        let mut s = String::new();
                        loop {
                            // easy return for next record
                            if let Some(line) = lines.peek() {
                                if line.starts_with("@") {
                                    break;
                                }
                            } else {
                                // file end
                                break;
                            }
                            // read lines for structure
                            if let Some(line) = lines.next() {
                                s.push_str(&format!("{}\n", line));
                            } else {
                                // file end
                                break;
                            }
                        }

                        let mol = Molecule::parse_from(s, "text/xyz")?;
                        results.molecule = Some(mol);
                    }
                    _ => {
                        warn!("ignored header: {:?}", header);
                    }
                }
            }
        } else {
            // file end
            break;
        }
    }

    Ok(results)
}

#[test]
fn test_model_parse_results() {
    let output = "@result_file_format_version
0.1
@structure
3
CH2
    C      7.50000000     11.59838500     11.36570800
    H     12.79336700     22.88608500     13.03115500
    H     25.95160100      9.92351500     13.03115500
@total_energy        R 1
 -0.32933619218901E+00
@dipole_moments      R 3
  0.00000000000000E+00  0.00000000000000E+00  0.00000000000000E+00
@forces              R 9
 -0.10525500903260E-03 -0.49341715067988E-04 -0.41578514351576E-04
 -0.10538090864721E-03 -0.44152035166424E-04 -0.46770596426924E-04
  0.11741769879568E-03 -0.22970049507365E-03 -0.64344681036487E-04
";

    let r = parse_model_results(output).expect("model results");
    assert!(r.molecule.is_some());
    assert_eq!(3, r.molecule.unwrap().natoms());

    let e = r.energy.expect("model result: energy");
    assert_relative_eq!(-0.329336, e, epsilon=1e-4);
}
// ea1864bb-6cc4-42f1-93f5-cebd790c58ab ends here
