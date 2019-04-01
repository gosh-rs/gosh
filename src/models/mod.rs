// imports

// [[file:~/Workspace/Programming/gosh/gosh.note::*imports][imports:1]]
use crate::core_utils::*;

use gchemol::prelude::*;
use gchemol::{
    io,
    Molecule
};
// imports:1 ends here

// mods

// [[file:~/Workspace/Programming/gosh/gosh.note::*mods][mods:1]]
pub mod blackbox;
pub mod lj;

pub use self::blackbox::BlackBox;
pub use self::lj::LennardJones;
// mods:1 ends here

// imports

// [[file:~/Workspace/Programming/gosh/gosh.note::*imports][imports:1]]
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
// imports:1 ends here

// base

// [[file:~/Workspace/Programming/gosh/gosh.note::*base][base:1]]
const MODEL_PROPERTIES_FORMAT_VERSION: &str = "0.1";

/// The computed results by external application
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelProperties {
    pub energy: Option<f64>,
    pub forces: Option<Vec<[f64; 3]>>,
    pub dipole: Option<[f64; 3]>,
    #[serde(skip_deserializing, skip_serializing)]
    pub molecule: Option<Molecule>,
    #[serde(skip_deserializing, skip_serializing)]
    pub force_constants: Option<Vec<[f64; 3]>>,
}
// base:1 ends here

// display/parse

// [[file:~/Workspace/Programming/gosh/gosh.note::*display/parse][display/parse:1]]
impl ModelProperties {
    /// Parse mulitple entries of ModelProperties from string slice
    pub fn parse_all(output: &str) -> Result<Vec<ModelProperties>> {
        parse_model_results(output)
    }

    /// Return true if there is no useful properties
    pub fn is_empty(&self) -> bool {
        //self.energy.is_none() && self.forces.is_none() && self.molecule.is_none()
        self.energy.is_none() && self.forces.is_none()
    }
}

impl fmt::Display for ModelProperties {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut txt = format!(
            "@model_properties_format_version {}\n",
            MODEL_PROPERTIES_FORMAT_VERSION
        );

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

impl FromStr for ModelProperties {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let all = parse_model_results(s)?;

        let n = all.len();
        if n == 0 {
            bail!("no valid results found!");
        }

        Ok(all[n - 1].clone())
    }
}

// parse a single entry of ModelProperties
fn parse_model_results_single(part: &[&str]) -> Result<ModelProperties> {
    // collect records as header separated lines
    // blank lines are ignored
    let mut records: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut header = None;
    for line in part {
        let line = line.trim();
        if line.starts_with("@") {
            header = line.split_whitespace().next();
        } else {
            if let Some(k) = header {
                records.entry(k).or_insert(vec![]).push(line);
            }
        }
    }

    // parse record values
    if records.len() < 1 {
        warn!("collected no results!");
    }

    let mut results = ModelProperties::default();
    for (k, lines) in records {
        match k {
            "@energy" => {
                assert_eq!(1, lines.len(), "expect one line containing energy");
                let energy = lines[0].trim().parse()?;
                results.energy = Some(energy);
            }
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
            }
            "@structure" => {
                let mut s = lines.join("\n");
                s.push_str("\n\n");
                let mol = Molecule::parse_from(s, "text/pxyz")?;
                results.molecule = Some(mol);
            }
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

fn parse_model_results(stream: &str) -> Result<Vec<ModelProperties>> {
    if stream.trim().is_empty() {
        bail!("Attemp to parse empty string!");
    }

    // ignore commenting lines or blank lines
    let lines: Vec<_> = stream
        .lines()
        .filter(|l| {
            let l = l.trim();
            !l.starts_with("#") && !l.is_empty()
        })
        .collect();

    let parts = lines[1..].split(|l| l.starts_with("@model_properties_format_version"));

    let mut all_results = vec![];
    for part in parts {
        // collect records as header separated lines
        // blank lines are ignored
        let mp = parse_model_results_single(part)?;
        all_results.push(mp);
    }

    Ok(all_results)
}
// display/parse:1 ends here

// test

// [[file:~/Workspace/Programming/gosh/gosh.note::*test][test:1]]
#[test]
fn test_model_parse_results() {
    use gchemol::io;
    use serde_json;

    let txt = io::read_file("tests/files/models/sample.txt").unwrap();
    let r: ModelProperties = txt.parse().expect("model results");

    // serializing
    let serialized = serde_json::to_string(&r).unwrap();
    // and deserializing
    let deserialized: ModelProperties = serde_json::from_str(&serialized).unwrap();

    // reformat
    let txt = format!("{}", r);

    // parse again
    let r: ModelProperties = txt.parse().expect("model results");

    assert!(&r.molecule.is_some());
    let ref mol = r.molecule.unwrap();
    assert_eq!(3, mol.natoms());
    let e = r.energy.expect("model result: energy");
    assert_relative_eq!(-0.329336, e, epsilon=1e-4);
}
// test:1 ends here

// chemical model

// [[file:~/Workspace/Programming/gosh/gosh.note::*chemical%20model][chemical model:1]]
pub trait ChemicalModel {
    /// Define how to compute molecular properties, such as energy, forces, ...
    fn compute(&self, mol: &Molecule) -> Result<ModelProperties>;

    /// Define how to compute the properties of many molecules in batch
    fn compute_many(&self, _mols: &[Molecule]) -> Result<Vec<ModelProperties>> {
        unimplemented!()
    }
}
// chemical model:1 ends here
