// [[file:~/Workspace/Programming/gosh/gosh.note::4982806b-0e81-4a97-b5f9-52f6abc5618a][4982806b-0e81-4a97-b5f9-52f6abc5618a]]
use errors::*;
use std::process::Command;

use gchemol::{
    self,
    Molecule,
    io,
    formats,
};

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
        self.molecules = io::read(filename).chain_err(|| "failed to load molecules")?;
        self.filename = Some(filename.to_owned());

        Ok(())
    }

    pub fn write(&self, filename: &str) -> Result<()> {
        if ! self.molecules.is_empty() {
            // mol.to_file(filename).chain_err(|| "failed to save molecule.")?;
            io::write(filename, &self.molecules).chain_err(|| "failed to save molecules.")?;
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

    pub fn clean(&mut self) -> Result<()> {
        if ! self.molecules.is_empty() {
            self.molecules[0].clean();
        } else {
            bail!("No molecule available.");
        }
        Ok(())
    }

    pub fn avail(&mut self) -> Result<()> {
        formats::describe_backends();
        Ok(())
    }

    pub fn rebond(&mut self) -> Result<()> {
        if ! self.molecules.is_empty() {
            self.molecules[0].rebond();
        } else {
            bail!("No molecule available.");
        }
        Ok(())
    }

    pub fn extern_cmdline(&self, cmdline: &str) -> Result<()> {
        let output = Command::new(cmdline)
            .output()
            .chain_err(|| format!("external cmdline failed: {}", cmdline))?;
        if output.status.success() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        } else {
            println!("{}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }
}
// 4982806b-0e81-4a97-b5f9-52f6abc5618a ends here
