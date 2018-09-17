// [[file:~/Workspace/Programming/gosh/gosh.note::894d0c1b-0482-46b9-a3dc-8f00b78833bc][894d0c1b-0482-46b9-a3dc-8f00b78833bc]]
use quicli::prelude::*;
use std::path::{Path, PathBuf};

use gchemol::prelude::*;
use gchemol::{
    io,
    Molecule
};

pub mod dftb;
pub mod lj;
// 894d0c1b-0482-46b9-a3dc-8f00b78833bc ends here

// [[file:~/Workspace/Programming/gosh/gosh.note::*chemical%20model][chemical model:1]]
pub trait ChemicalModel {
    /// define how to calculate properties, such as energy, forces, ...
    fn compute(&self, mol: &Molecule) -> Result<ModelResults>;

    fn positions(&self) -> Vec<[f64; 3]> {
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
}
// chemical model:1 ends here

// [[file:~/Workspace/Programming/gosh/gosh.note::*display/parse][display/parse:1]]
use std::fmt;
use std::str::FromStr;
use std::collections::HashMap;

const MODEL_RESULTS_FORMAT_VERSION: &str = "0.1";

/// The computed results by external application
#[derive(Debug, Clone, Default)]
pub struct ModelResults {
    pub molecule        : Option<Molecule>,
    pub energy          : Option<f64>,
    pub forces          : Option<Vec<[f64; 3]>>,
    pub dipole          : Option<[f64; 3]>,
    pub force_constants : Option<Vec<[f64; 3]>>,
    // polarizability
    // dipole_derivatives
}

impl fmt::Display for ModelResults {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut txt = format!("@model_results_format_version\n{}\n", MODEL_RESULTS_FORMAT_VERSION);

        // structure
        if let Some(mol) = &self.molecule {
            txt.push_str("@structure\n");
            let coords = mol.format_as("text/pxyz").expect("formatted molecule");
            txt.push_str(&coords);
        }
        // energy
        if let Some(energy) = &self.energy {
            txt.push_str("@energy\n");
            txt.push_str(&format!("{:-20.12E}\n", energy));
        }
        // forces
        if let Some(forces) = &self.forces {
            txt.push_str("@forces\n");
            for [fx, fy, fz] in forces {
                let line = format!("{:-20.12E} {:-20.12E} {:-20.12E}\n", fx, fy, fz);
                txt.push_str(&line);
            }
        }

        // dipole moments
        if let Some(d) = &self.dipole {
            txt.push_str("@dipole\n");
            let line = format!("{:-20.12E} {:-20.12E} {:-20.12E}\n", d[0], d[1], d[2]);
            txt.push_str(&line);
        }

        write!(f, "{}", txt)
    }
}

impl FromStr for ModelResults {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        parse_model_results(s)
    }
}

fn parse_model_results(stream: &str) -> Result<ModelResults>{
    let mut results = ModelResults::default();

    // ignore commenting lines or blank lines
    let mut lines = stream.lines()
        .filter(|l| {
            let l = l.trim();
            ! l.starts_with("#") && ! l.is_empty()
        })
        .peekable();

    // collect records as header separated lines
    // blank lines are ignored
    let mut records: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut header = None;
    for line in lines {
        let line = line.trim();
        if line.starts_with("@") {
            header = line.split_whitespace().next();
        } else {
            if let Some(k) = header{
                records
                    .entry(k)
                    .or_insert(vec![])
                    .push(line);
            }
        }
    }

    // parse record values
    if records.len() < 1 {
        warn!("collected no results!");
    }

    for (k, lines) in records {
        match k {
            "@energy" => {
                assert_eq!(1, lines.len(), "expect one line containing energy");
                let energy = lines[0].trim().parse()?;
                results.energy = Some(energy);
            },
            "@forces" => {
                let mut forces: Vec<[f64; 3]> = vec![];
                for line in lines {
                    let parts: Vec<_> = line.split_whitespace().collect();
                    if parts.len() != 3 {
                        bail!("expect xyz forces: {}", line);
                    }
                    let fx = parts[0].parse()?;
                    let fy = parts[1].parse()?;
                    let fz = parts[2].parse()?;
                    forces.push([fx, fy, fz]);
                }

                results.forces = Some(forces);
            },
            "@structure" => {
                let mut s = lines.join("\n");
                s.push_str("\n\n");
                let mol = Molecule::parse_from(s, "text/pxyz")?;
                results.molecule = Some(mol);
            },
            "@dipole" => {
                assert_eq!(1, lines.len(), "expect one line containing dipole moment");
                let parts: Vec<_> = lines[0].split_whitespace().collect();
                let fx = parts[0].parse()?;
                let fy = parts[1].parse()?;
                let fz = parts[2].parse()?;
                results.dipole = Some([fx, fy, fz]);
            }
            _ => {
                warn!("ignored record: {:?}", k);
            }
        }
    }

    Ok(results)
}
// display/parse:1 ends here

// [[file:~/Workspace/Programming/gosh/gosh.note::*test][test:1]]
#[test]
fn test_model_parse_results() {
    use gchemol::io;

    let txt = io::read_file("tests/files/model_results/sample.txt").unwrap();
    let r = parse_model_results(&txt).expect("model results");

    // reformat
    let txt = format!("{}", r);

    // parse again
    let r = parse_model_results(&txt).expect("model results");

    assert!(&r.molecule.is_some());
    let ref mol = r.molecule.unwrap();
    assert_eq!(3, mol.natoms());
    let e = &r.energy.expect("model result: energy");
    assert_relative_eq!(-0.329336, e, epsilon=1e-4);
}
// test:1 ends here
