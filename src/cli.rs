// [[file:~/Workspace/Programming/gosh/gosh.note::4982806b-0e81-4a97-b5f9-52f6abc5618a][4982806b-0e81-4a97-b5f9-52f6abc5618a]]
use errors::*;

use gchemol::{
    Molecule,
};

/// A commander for interactive interpreter
pub struct Commander {
    /// active molecule
    pub molecule: Option<Molecule>,
    /// input file containg molecules
    pub filename: Option<String>,
}

impl Commander {
    pub fn new() -> Self {
        Commander {
            molecule: None,
            filename: None,
        }
    }

    pub fn load(&mut self, filename: &str) -> Result<()> {
        let mol = Molecule::from_file(filename).chain_err(|| "bad")?;
        self.molecule = Some(mol);
        self.filename = Some(filename.to_owned());

        Ok(())
    }

    pub fn write(&self, filename: &str) -> Result<()> {
        if let Some(ref mol) = self.molecule {
            mol.to_file(filename).chain_err(|| "failed to save molecule.")?;
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
        if let Some(ref mut mol) = &mut self.molecule {
            mol.clean();
        } else {
            bail!("No molecule available.");
        }
        Ok(())
    }

    pub fn rebond(&mut self) -> Result<()> {
        if let Some(ref mut mol) = &mut self.molecule {
            mol.clean();
        } else {
            bail!("No molecule available.");
        }
        Ok(())
    }
}
// 4982806b-0e81-4a97-b5f9-52f6abc5618a ends here
