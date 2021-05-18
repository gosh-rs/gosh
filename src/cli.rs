// [[file:../gosh.note::*imports][imports:1]]
use crate::core::*;
use crate::gchemol;

use gchemol::prelude::*;
use gchemol::{io, Molecule};

use gut::cli::*;
use gut::prelude::*;

use std::path::PathBuf;
use std::process::Command;
use structopt::*;

type CliResult = Result<()>;
// imports:1 ends here

// [[file:../gosh.note::*base][base:1]]
/// A commander for interactive interpreter
pub struct Commander {
    /// active molecules
    pub molecules: Vec<Molecule>,
    /// input file containg molecules
    pub filename: Option<PathBuf>,
}

#[derive(StructOpt, Debug)]
#[structopt(setting = structopt::clap::AppSettings::VersionlessSubcommands)]
pub enum GoshCmd {
    /// Quit go shell.
    #[structopt(name = "quit", alias = "q", alias = "exit")]
    Quit {},

    /// Show available commands.
    #[structopt(name = "help", alias = "h", alias = "?")]
    Help {},

    /// Write molecule(s) into file.
    #[structopt(name = "write", alias = "save")]
    Write {
        /// The filename to write.
        #[structopt(name = "FILE-NAME")]
        filename: Option<PathBuf>,
    },

    /// Load molecule(s) from file.
    #[structopt(name = "load")]
    Load {
        /// The filename containing one or more molecules.
        #[structopt(name = "MOLECULE-NAME")]
        filename: PathBuf,
    },

    /// Load molecule from checkpoint file.
    #[structopt(name = "load-chk")]
    LoadChk {
        /// The filename containing one or more molecules.
        #[structopt(name = "MOLECULE-NAME")]
        filename: PathBuf,

        #[structopt(long, default_value = "-1")]
        chk_slot: i32,
    },

    /// Rebuild bonds based on atom distances.
    #[structopt(name = "rebond")]
    Rebond {
        #[structopt(short = "-r")]
        /// The bonding ratio for guessing chemical bonds. Larger value leading
        /// to more bonds. The default value is 0.55
        bonding_ratio: Option<f64>,
    },

    /// Clean up bad molecular geometry.
    #[structopt(name = "clean")]
    Clean {},

    /// Unbuild current crystal structure leaving a non-periodic structure.
    #[structopt(name = "unbuild_crystal")]
    UnbuildCrystal {},

    /// Convert molecule formats in batch.
    ///
    /// Usage: convert 1.xyz 2.xyz -e .mol2
    #[structopt(name = "convert")]
    Convert {
        /// input files: e.g.: 1.cif 2.cif 3.cif
        files: Vec<PathBuf>,
        /// target format (file extension): e.g.: .mol2 or .poscar
        #[structopt(short = "-e")]
        format_to: String,
    },

    /// Format molecule using template file.
    #[structopt(name = "format")]
    Format {
        /// Path to template file.
        #[structopt(name = "TEMPLATE_NAME", parse(from_os_str))]
        filename: PathBuf,

        /// Path to output file.
        #[structopt(name = "OUTPUT_FILE_NAME", short = "-o")]
        output: Option<PathBuf>,
    },

    /// Break molecule into smaller fragments based on connectivity.
    #[structopt(name = "fragment")]
    Fragment {},

    /// Create supercell for all loaded molecules.
    #[structopt(name = "supercell")]
    Supercell {
        /// range a
        range_a: usize,
        /// range b
        range_b: usize,
        /// range c
        range_c: usize,
    },

    /// Superimpose current molecule onto reference molecule by translating and
    /// rotating target molecule
    #[structopt(name = "superimpose")]
    Superimpose {
        /// Path to reference molecule file.
        #[structopt(name = "REFERENCE_MOLECULE", parse(from_os_str))]
        filename: PathBuf,
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

// [[file:../gosh.note::*core][core:1]]
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
                self.molecules = gchemol::io::read_all(filename)?;
                self.filename = Some(filename.to_owned());

                println!("Loaded {} molecule(s).", self.molecules.len());
            }

            GoshCmd::LoadChk { filename, chk_slot } => {
                let chk = gosh_database::CheckpointDb::new(&filename);
                let mol: Molecule = chk.load_from_slot_n(*chk_slot)?;

                self.molecules = vec![mol];
                self.filename = Some(filename.to_owned());

                println!("Loaded one molecule from checkpoint file: {}", filename.display());
            }

            GoshCmd::Clean {} => {
                self.check()?;
                for i in 0..self.molecules.len() {
                    self.molecules[i].clean()?;
                }
            }

            GoshCmd::Avail {} => {
                gchemol::io::describe_backends();
            }
            GoshCmd::UnbuildCrystal {} => {
                self.check()?;
                for i in 0..self.molecules.len() {
                    self.molecules[i].unbuild_crystal();
                }
            }

            GoshCmd::Write { filename } => {
                self.check()?;

                if let Some(filename) = filename.as_ref().or(self.filename.as_ref()) {
                    io::write(&filename, &self.molecules)?;
                    println!("Wrote {} molecules in {}", self.molecules.len(), filename.display());
                } else {
                    eprintln!("No filename.");
                }
            }

            #[cfg(feature = "adhoc")]
            GoshCmd::Fragment {} => {
                // FIXME: remove
                use gchemol::compat::*;

                self.check()?;
                let mols = self.molecules[0].fragment();
                self.molecules.clear();
                self.molecules.extend(mols);
            }

            GoshCmd::Rebond { bonding_ratio } => {
                self.check()?;
                if let Some(r) = bonding_ratio {
                    std::env::set_var("GCHEMOL_BONDING_RATIO", format!("{}", r));
                }
                for mol in self.molecules.iter_mut() {
                    mol.rebond();
                }
            }
            GoshCmd::Convert { files, format_to } => {
                for f in files.iter() {
                    let mols = gchemol::io::read_all(f)?;
                    io::write(f.with_extension(&format_to[1..]), &mols)?;
                }
                println!("{} files were converted in {} format", files.len(), format_to);
            }
            GoshCmd::Supercell {
                range_a,
                range_b,
                range_c,
            } => {
                self.check()?;

                let mut mols = vec![];
                for mol in self.molecules.iter() {
                    if let Some(mol) = mol.supercell(*range_a, *range_b, *range_c) {
                        mols.push(mol);
                    } else {
                        eprintln!("No lattice data.");
                    }
                }
                self.molecules = mols;
            }
            GoshCmd::Superimpose { filename } => {
                self.check()?;
                todo!()
            }
            GoshCmd::Format { filename, output } => {
                self.check()?;

                let mut ss = vec![];
                for mol in &self.molecules {
                    let s = mol
                        .render_with(&filename)
                        .with_context(|| format!("Failed to render molecule with file: {:?}", filename))?;
                    ss.push(s);
                }

                let s = ss.join("");
                if let Some(output) = output {
                    gut::fs::write_to_file(output, &s)?;
                } else {
                    println!("{:}", s);
                }
            }
            o => {
                eprintln!("{:?}: not implemented yet!", o);
            }
        }

        Ok(())
    }

    /// basic sanity check
    fn check(&self) -> Result<()> {
        if self.molecules.is_empty() {
            bail!("No active molecule available.")
        }

        Ok(())
    }
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
// core:1 ends here
