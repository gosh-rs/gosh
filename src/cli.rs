// [[file:~/Workspace/Programming/gosh/gosh.note::4982806b-0e81-4a97-b5f9-52f6abc5618a][4982806b-0e81-4a97-b5f9-52f6abc5618a]]
use quicli::prelude::*;
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
        self.molecules = io::read(filename).map_err(|e| format_err!("failed to load molecules"))?;
        self.filename = Some(filename.to_owned());

        Ok(())
    }

    pub fn write(&self, filename: &str) -> Result<()> {
        if ! self.molecules.is_empty() {
            io::write(filename, &self.molecules).map_err(|e| format_err!("failed to save molecules."))?;
        } else {
            bail!("No active molecule available.");
        }

        Ok(())
    }

    pub fn format(&self, template_file: &str) -> Result<()> {
        if ! self.molecules.is_empty() {
            let mol = &self.molecules[0];
            let template = io::read_file(template_file)
                .map_err(|e| format_err!("failed to load template"))?;
            let s = formats::template::render_molecule_with(mol, &template)
                .map_err(|e| format_err!("failed to render molecule"))?;
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

    pub fn fragment(&mut self) -> Result<()> {
        if ! self.molecules.is_empty() {
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
            .map_err(|e| format_err!("external cmdline failed: {}", cmdline))?;
        if output.status.success() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        } else {
            println!("{}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }
}
// 4982806b-0e81-4a97-b5f9-52f6abc5618a ends here
