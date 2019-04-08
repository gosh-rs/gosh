// cmd

// [[file:~/Workspace/Programming/gosh-rs/gosh/gosh.note::*cmd][cmd:1]]
use gosh::core_utils::*;
use std::process::Command;

use gchemol::prelude::*;
use gchemol::{io, Molecule};

/// A commander for interactive interpreter
pub struct Commander {
    /// active molecules
    pub molecules: Vec<Molecule>,
    /// input file containg molecules
    pub filename: Option<String>,
}

impl Commander {
    pub fn new() -> Self {
        Commander {
            filename: None,
            molecules: vec![],
        }
    }

    pub fn load(&mut self, filename: &str) -> Result<()> {
        self.molecules = io::read(filename).map_err(|_| format_err!("failed to load molecules"))?;
        self.filename = Some(filename.to_owned());

        Ok(())
    }

    pub fn write(&self, filename: &str) -> Result<()> {
        if !self.molecules.is_empty() {
            io::write(filename, &self.molecules)
                .map_err(|_| format_err!("failed to save molecules."))?;
        } else {
            bail!("No active molecule available.");
        }

        Ok(())
    }

    pub fn format(&self, template_file: &str) -> Result<()> {
        if !self.molecules.is_empty() {
            let mol = &self.molecules[0];
            let template =
                io::read_file(template_file).map_err(|_| format_err!("failed to load template"))?;
            let s = mol
                .render_with(&template)
                .map_err(|_| format_err!("failed to render molecule"))?;
            println!("{:}", s);
        } else {
            bail!("No active molecule available.");
        }

        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        if let Some(ref filename) = self.filename {
            self.write(filename)?;
        } else {
            bail!("Don't known where to save file.");
        }

        Ok(())
    }

    /// Create supercell for molecules.
    pub fn supercell(&mut self, range_txt: &str) -> Result<()> {
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
            bail!("No molecule available.");
        }

        Ok(())
    }

    pub fn clean(&mut self) -> Result<()> {
        if !self.molecules.is_empty() {
            self.molecules[0].clean()?;
        } else {
            bail!("No molecule available.");
        }
        Ok(())
    }

    pub fn avail(&mut self) -> Result<()> {
        // formats::describe_backends();
        Ok(())
    }

    pub fn rebond(&mut self) -> Result<()> {
        if !self.molecules.is_empty() {
            for mol in self.molecules.iter_mut() {
                mol.rebond();
            }
        } else {
            bail!("No molecule available.");
        }
        Ok(())
    }

    pub fn fragment(&mut self) -> Result<()> {
        if !self.molecules.is_empty() {
            let mols = self.molecules[0].fragment();
            self.molecules.clear();
            self.molecules.extend(mols);
        } else {
            bail!("No molecule available.");
        }
        Ok(())
    }

    pub fn extern_cmdline(&self, cmdline: &str) -> Result<()> {
        let output = Command::new(cmdline)
            .output()
            .map_err(|_| format_err!("external cmdline failed: {}", cmdline))?;
        if output.status.success() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        } else {
            println!("{}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }
}
// cmd:1 ends here
