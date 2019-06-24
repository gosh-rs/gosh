// imports

// [[file:~/Workspace/Programming/gosh-rs/gosh/gosh.note::*imports][imports:1]]
use crate::core_utils::*;
use crate::cmd_utils::*;

use gchemol::prelude::*;
use gchemol::{io, Molecule};
use std::process::Command;
use std::path::PathBuf;
// imports:1 ends here

// cmd

// [[file:~/Workspace/Programming/gosh-rs/gosh/gosh.note::*cmd][cmd:1]]
/// A commander for interactive interpreter
pub struct Commander {
    /// active molecules
    pub molecules: Vec<Molecule>,
    /// input file containg molecules
    pub filename: Option<PathBuf>,
}

fn run_cmd(cmdline: &str) -> CliResult {
    let output = std::process::Command::new(cmdline)
        .output()
        .map_err(|_| format_err!("external cmdline failed: {}", cmdline))?;

    if output.status.success() {
        println!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

impl Commander {
    pub fn new() -> Self {
        Commander {
            filename: None,
            molecules: vec![],
        }
    }

    pub fn action(&mut self, cmd: &GoshCmd) -> CliResult {
        match cmd {
            GoshCmd::Quit {} | GoshCmd::Help {} => {
                //
            }
            GoshCmd::List {} => {
                if let Err(ref e) = run_cmd("ls") {
                    eprintln!("{:?}", e);
                }
            }
            GoshCmd::Pwd {} => {
                if let Err(ref e) = run_cmd("pwd") {
                    eprintln!("{:?}", e);
                }
            }
            GoshCmd::Load { filename } => {
                self.molecules =
                    io::read(filename).map_err(|_| format_err!("failed to load molecules"))?;
                self.filename = Some(filename.to_owned());
            }

            GoshCmd::Clean {} => {
                if !self.molecules.is_empty() {
                    self.molecules[0].clean()?;
                } else {
                    eprintln!("No molecule available.");
                }
            }
            GoshCmd::Avail {} => {
                gchemol::io::describe_backends();
            }
            GoshCmd::Write { filename } => {
                if !self.molecules.is_empty() {
                    io::write(filename, &self.molecules)
                        .map_err(|_| format_err!("failed to save molecules."))?;
                } else {
                    eprintln!("No active molecule available.");
                }
            }
            GoshCmd::Fragment {} => {
                if !self.molecules.is_empty() {
                    let mols = self.molecules[0].fragment();
                    self.molecules.clear();
                    self.molecules.extend(mols);
                } else {
                    eprintln!("No molecule available.");
                }
            }
            GoshCmd::Rebond {} => {
                if !self.molecules.is_empty() {
                    for mol in self.molecules.iter_mut() {
                        mol.rebond();
                    }
                } else {
                    eprintln!("No molecule available.");
                }
            }
            GoshCmd::Supercell { range_txt } => {
                use gchemol::Supercell;

                if !self.molecules.is_empty() {
                    let range: Vec<isize> = range_txt
                        .split_whitespace()
                        .map(|s| s.parse().expect("integer"))
                        .collect();

                    assert!(range.len() >= 3, "wrong number of sizes.");
                    let mut mols = vec![];
                    for mol in self.molecules.iter() {
                        let mol = Supercell::new()
                            .with_range_a(0, range[0])
                            .with_range_b(0, range[1])
                            .with_range_c(0, range[2])
                            .build(&mol);
                        mols.push(mol);
                    }
                    self.molecules = mols;
                } else {
                    eprintln!("No molecule available.");
                }
            }
            GoshCmd::Format { filename } => {
                if !self.molecules.is_empty() {
                    let mol = &self.molecules[0];
                    let template = io::read_file(&filename)
                        .map_err(|_| format_err!("failed to load template"))?;
                    let s = mol
                        .render_with(&template)
                        .map_err(|_| format_err!("failed to render molecule"))?;
                    println!("{:}", s);
                } else {
                    eprintln!("No active molecule available.");
                }
            }
            o => {
                eprintln!("{:?}: not implemented yet!", o);
            }
        }

        Ok(())
    }
}
// cmd:1 ends here

// base

// [[file:~/Workspace/Programming/gosh-rs/gosh/gosh.note::*base][base:1]]
#[derive(StructOpt, Debug)]
#[structopt(name = "gosh", about = "the stupid content tracker")]
pub enum GoshCmd {
    /// Quit go shell.
    #[structopt(name = "quit", alias = "q", alias = "exit")]
    Quit {},

    /// Show available commands.
    #[structopt(name = "help", alias = "h", alias = "?")]
    Help {},

    /// Write molecule(s) into file.
    #[structopt(name = "write")]
    Write {
        /// The filename to write.
        #[structopt(name = "FILE_NAME", parse(from_os_str))]
        filename: PathBuf,
    },

    /// Load molecule(s) from file.
    #[structopt(name = "load")]
    Load {
        /// The filename containing one or more molecules.
        #[structopt(name = "MOLECULE_NAME", parse(from_os_str))]
        filename: PathBuf,
    },

    /// Rebuild bonds based on atom distances.
    #[structopt(name = "rebond")]
    Rebond {},

    /// Clean up bad molecular geometry.
    #[structopt(name = "clean")]
    Clean {},

    /// Format molecule using user defined template file.
    #[structopt(name = "format")]
    Format {
        /// Path to template file.
        #[structopt(name = "TEMPLATE_NAME", parse(from_os_str))]
        filename: PathBuf,
    },

    /// Break molecule into smaller fragments based on connectivity.
    #[structopt(name = "fragment")]
    Fragment {},

    /// Create supercell for all loaded molecules.
    #[structopt(name = "supercell")]
    Supercell {
        /// range text
        range_txt: String
    },

    /// Show supported file formats.
    #[structopt(name = "avail")]
    Avail {},

    /// List files under current directory.
    #[structopt(name = "ls", alias = "l", alias = "ll")]
    List {},

    /// Print path to current directory.
    #[structopt(name = "pwd")]
    Pwd {},
}
// base:1 ends here
