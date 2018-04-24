// [[file:~/Workspace/Programming/gosh/gosh.note::4982806b-0e81-4a97-b5f9-52f6abc5618a][4982806b-0e81-4a97-b5f9-52f6abc5618a]]
use errors::*;

use gchemol::{
    Molecule,
};

/// A commander for interactive interpreter
pub struct Commander {
    pub molecule: Option<Molecule>,
}

impl Commander {
    pub fn new() -> Self {
        Commander {
            molecule: None,
        }
    }

    pub fn load(&mut self, filename: &str) -> Result<()> {
        let mol = Molecule::from_file(filename).chain_err(|| "bad")?;
        self.molecule = Some(mol);

        Ok(())
    }
}
// 4982806b-0e81-4a97-b5f9-52f6abc5618a ends here
