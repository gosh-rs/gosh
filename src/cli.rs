// [[file:../gosh.note::*imports][imports:1]]
use crate::common::*;

use gchemol::prelude::*;
use gchemol::{io, Molecule};

use clap::{AppSettings, Clap};

use std::path::PathBuf;
use std::process::Command;

use gut::utils::parse_numbers_human_readable;
// imports:1 ends here

// [[file:../gosh.note::*base][base:1]]
/// A commander for interactive interpreter
pub struct Commander {
    /// active molecules
    pub molecules: Vec<Molecule>,
    /// input file containg molecules
    pub filename: Option<PathBuf>,
}

#[derive(Clap, Debug)]
#[clap(setting = AppSettings::ColoredHelp)]
#[clap(setting = AppSettings::VersionlessSubcommands)]
pub enum GoshCmd {
    /// Quit go shell.
    #[clap(name = "quit", alias = "q", alias = "exit")]
    Quit {},

    /// Show available commands.
    #[clap(name = "help", alias = "h", alias = "?")]
    Help {},

    /// Write molecule(s) into file.
    #[clap(name = "write", alias = "save")]
    Write {
        /// The filename to write.
        #[clap(name = "FILE-NAME")]
        filename: Option<PathBuf>,
    },

    /// Load molecule(s) from file.
    #[clap(name = "load")]
    Load {
        /// The filename containing one or more molecules.
        #[clap(name = "MOLECULE-NAME")]
        filename: PathBuf,
    },

    /// Load molecule from checkpoint file.
    #[clap(name = "load-chk")]
    LoadChk {
        /// The filename containing one or more molecules.
        #[clap(name = "MOLECULE-NAME")]
        filename: PathBuf,

        #[clap(long, default_value = "-1")]
        chk_slot: i32,
    },

    /// Rebuild bonds based on atom distances.
    #[clap(name = "rebond")]
    Rebond {
        #[clap(short = 'r')]
        /// The bonding ratio for guessing chemical bonds. Larger value leading
        /// to more bonds. The default value is 0.55
        bonding_ratio: Option<f64>,
    },

    /// Update current molecule from somewhere with something
    #[clap(name = "update")]
    Update {
        /// The target properties to be updated: coords, freezing, ...
        target: String,
        
        #[clap(short = 's')]
        /// Select the atoms to be updated: "2,3,8" or "2-9"
        select: Option<String>,

        #[clap(short = 'f')]
        /// The path to source molecular file
        source: PathBuf,
    },

    /// Clean up bad molecular geometry.
    #[clap(name = "clean")]
    Clean {},

    /// Unbuild current crystal structure leaving a non-periodic structure.
    #[clap(name = "unbuild_crystal")]
    UnbuildCrystal {},

    /// Convert molecule formats in batch.
    ///
    /// Usage: convert 1.xyz 2.xyz -e .mol2
    #[clap(name = "convert")]
    Convert {
        /// input files: e.g.: 1.cif 2.cif 3.cif
        files: Vec<PathBuf>,
        /// target format (file extension): e.g.: .mol2 or .poscar
        #[clap(short = 'e')]
        format_to: String,
    },

    /// Format molecule using template file.
    #[clap(name = "format")]
    Format {
        /// Path to template file.
        #[clap(name = "TEMPLATE_NAME", parse(from_os_str))]
        filename: PathBuf,

        /// Path to output file.
        #[clap(name = "OUTPUT_FILE_NAME", short = 'o')]
        output: Option<PathBuf>,
    },

    /// Break molecule into smaller fragments based on connectivity.
    #[clap(name = "fragment")]
    Fragment {},

    /// Create supercell for all loaded molecules.
    #[clap(name = "supercell")]
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
    #[clap(name = "superimpose")]
    Superimpose {
        /// Path to reference molecule file.
        #[clap(name = "REFERENCE_MOLECULE", parse(from_os_str))]
        filename: PathBuf,
    },

    /// Show supported file formats.
    #[clap(name = "avail")]
    Avail {},

    /// List files under current directory.
    #[clap(name = "ls", alias = "l", alias = "ll")]
    List {},

    /// Print path to current directory.
    #[clap(name = "pwd")]
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

    pub fn action(&mut self, cmd: &GoshCmd) -> Result<()> {
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
                let filename = normalize_path(&filename);
                self.molecules = gchemol::io::read_all(&filename)?;
                self.filename = filename.into();

                println!("Loaded {} molecule(s).", self.molecules.len());
            }

            GoshCmd::LoadChk { filename, chk_slot } => {
                let filename = normalize_path(&filename);

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
                    let filename = normalize_path(filename);
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
                let filename = normalize_path(filename);

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
            // FIXME: rewrite
            GoshCmd::Update { target, select, source } => {
                self.check()?;
                if self.molecules.len() != 1 {
                    bail!("only work for a single molecule");
                }
                let mol = Molecule::from_file(&source)?;
                if mol.natoms() != self.molecules[0].natoms() {
                    bail!("invalid source!");
                }

                match target.as_str() {
                    "coords" => {
                        let m: std::collections::HashMap<_, _> = if let Some(select) = select {
                            let selected_atoms = parse_numbers_human_readable(&select)?;
                            mol.atoms()
                                .filter_map(|(i, a)| {
                                    if selected_atoms.contains(&i) {
                                        Some((i, a.position()))
                                    } else {
                                        None
                                    }
                                })
                                .collect()
                        } else {
                            mol.atoms().map(|(i, a)| (i, a.position())).collect()
                        };
                        for (i, p) in m {
                            let a = self.molecules[0].get_atom_mut(i).unwrap();
                            a.set_position(p);
                        }
                    }
                    "freezing" => {
                        let m: std::collections::HashMap<_, _> = if let Some(select) = select {
                            let selected_atoms = parse_numbers_human_readable(&select)?;
                            mol.atoms()
                                .filter_map(|(i, a)| {
                                    if selected_atoms.contains(&i) {
                                        Some((i, a.freezing()))
                                    } else {
                                        None
                                    }
                                })
                                .collect()
                        } else {
                            mol.atoms().map(|(i, a)| (i, a.freezing())).collect()
                        };
                        for (i, f) in m {
                            let a = self.molecules[0].get_atom_mut(i).unwrap();
                            a.set_freezing(f);
                        }
                    }
                    _ => {
                        bail!("update: not implemented yet!")
                    }
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

fn run_cmd(cmdline: &str) -> Result<()> {
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

fn normalize_path(s: &Path) -> PathBuf {
    // fix tilde char for HOME directory
    if s.starts_with("~") {
        if let Ok(h) = std::env::var("HOME") {
            let s = s.to_string_lossy();
            let s = s.replacen("~", &h, 1);
            return PathBuf::from(s);
        }
    }
    s.into()
}
// core:1 ends here
