// [[file:~/Workspace/Programming/gosh/gosh.note::894d0c1b-0482-46b9-a3dc-8f00b78833bc][894d0c1b-0482-46b9-a3dc-8f00b78833bc]]
use quicli::prelude::*;

#[derive(Debug, Clone)]
pub struct ModelResults {
    energy          : Option<f64>,
    forces          : Option<Vec<[f64; 3]>>,
    dipole_moment   : Option<[f64; 3]>,
    force_constants : Option<Vec<[f64; 3]>>,
}

impl Default for ModelResults {
    fn default() -> Self {
        ModelResults {
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
    fn calculate(&self) -> Result<()> {
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

    let mut lines = stream.lines();
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
@number_of_atoms I 1
3
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
    let e = r.energy.expect("model result: energy");
    assert_relative_eq!(-0.329336, e, epsilon=1e-4);
}
// ea1864bb-6cc4-42f1-93f5-cebd790c58ab ends here
