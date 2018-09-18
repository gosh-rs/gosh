// [[file:~/Workspace/Programming/gosh/gosh.note::*dftb.rs][dftb.rs:1]]
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
pub fn run<P: Into<PathBuf>>(mol: &Molecule, runfile: P) -> Result<ModelProperties> {
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

    let x: ModelProperties = output.parse()?;

    Ok(x)
}

impl ChemicalModel for DftbModel {
    fn compute(&self, mol: &Molecule) -> Result<ModelProperties> {
        run(&mol, &self.runfile)
    }
}
// dftb.rs:1 ends here
