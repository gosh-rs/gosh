// [[file:~/Workspace/Programming/gosh/gosh.note::fdd244b4-5403-411a-8cc9-d3782769762e][fdd244b4-5403-411a-8cc9-d3782769762e]]
use super::*;

pub struct DftbModel {
    runfile: PathBuf,
}

impl Default for DftbModel {
    fn default() -> Self {
        DftbModel {
            runfile: "./submit.sh".into(),
        }
    }
}

// perform dftb+ calculations
pub fn run<P: Into<PathBuf>>(mol: &Molecule, runfile: P) -> Result<ModelResults> {
    let runfile = runfile.into();
    let txt = mol.format_as("dftb/input")?;

    info!("run script file: {}", &runfile.display());

    // goto script parent directory
    let d = &runfile.parent().expect("run script parent dir");
    let output = cmd!(&runfile)
        .dir(d)
        .input(txt)
        .read()
        .map_err(|e| format_err!("{:?}: {:?}",
                                 &runfile.display(),
                                 e)
        )?;

    let x: ModelResults = output.parse()?;

    Ok(x)
}

impl ChemicalModel for DftbModel {
    fn calculate(&self, mol: &Molecule) -> Result<ModelResults> {
        run(&mol, &self.runfile)
    }
}
// fdd244b4-5403-411a-8cc9-d3782769762e ends here
