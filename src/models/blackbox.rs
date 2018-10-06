// base
// Represents an universal blackbox (external) model defined by user scripts

// [[file:~/Workspace/Programming/gosh/gosh.note::*base][base:1]]
use super::*;

pub struct BlackBox {
    pub runfile: PathBuf,
    pub tplfile: PathBuf,
}

impl Default for BlackBox {
    fn default() -> Self {
        BlackBox {
            runfile: "submit.sh".into(),
            tplfile: "input.hbs".into(),
        }
    }
}
// base:1 ends here

// render

// [[file:~/Workspace/Programming/gosh/gosh.note::*render][render:1]]
impl BlackBox {
    /// Render input using template
    pub fn render_input(&self, mol: &Molecule) -> Result<String> {
        // 1. load input template
        let template = io::read_file(&self.tplfile)
            .map_err(|e| format_err!("failed to load template:\n {}", e))?;

        // 2. render input text with the template
        let txt = mol.render_with(&template)?;

        Ok(txt)
    }
}
// render:1 ends here

// scratch

// [[file:~/Workspace/Programming/gosh/gosh.note::*scratch][scratch:1]]
use tempfile::{tempdir, tempdir_in, TempDir};

/// Create a temporary directory for scratch files. User can change the scratch
/// root directory by setting BBM_SCR_DIR environment variable.
pub fn new_scrdir() -> Result<TempDir> {
    use std::env;

    match env::var("BBM_SCR_DIR") {
        Ok(scr_root) => {
            info!("set scratch root directory as: {:?}", scr_root);
            Ok(tempdir_in(scr_root)?)
        },
        Err(err) => {
            debug!("scratch root is not set");
            Ok(tempdir()?)
        }
    }
}
// scratch:1 ends here

// dotenv

// [[file:~/Workspace/Programming/gosh/gosh.note::*dotenv][dotenv:1]]
use dotenv;
use std::env;
use std::path::{Path, PathBuf};
use quicli::prelude::*;

/// Enter directory with environment variables from .env file
fn enter_dir_with_env(dir: &Path) -> Result<()>{
    info!("read dotenv vars from {}", dir.display());

    // change to directory
    // env::set_current_dir(&dir)?;

    // read environment variables
    dotenv::from_path(&dir.join(".env")).ok();

    for (key, value) in env::vars() {
        if key.starts_with("BBM") {
            info!("{}: {}", key, value);
        }
    }

    Ok(())
}

impl BlackBox {
    /// Initialize from environment variables
    /// # Panic
    /// - Panic if the directory is inaccessible.
    pub fn from_dotenv<P: AsRef<Path>>(dir: P) -> Self {
        let dir = dir.as_ref();

        match enter_dir_with_env(dir) {
            Ok(_) => {},
            Err(e) => {
                warn!("dotenv failed: {:?}", e);
            }
        }

        let mut ropt = BlackBox::default();
        if let Ok(f) = env::var("BBM_RUN_FILE") {
            ropt.runfile = dir.join(f);
        } else {
            ropt.runfile = dir.join("submit.sh");
        }

        if let Ok(f) = env::var("BBM_TPL_FILE") {
            ropt.tplfile = dir.join(f);
        } else {
            ropt.tplfile = dir.join("input.hbs");
        }

        ropt
    }
}
// dotenv:1 ends here

// chemical model

// [[file:~/Workspace/Programming/gosh/gosh.note::*chemical%20model][chemical model:1]]
impl ChemicalModel for BlackBox {
    fn compute(&self, mol: &Molecule) -> Result<ModelProperties> {
        // 1. render input text with the template
        let txt = self.render_input(&mol)?;

        // 2. call external engine
        let output = safe_call(&self.runfile, &txt)?;

        // 3. collect model properties
        let p: ModelProperties = output.parse()?;

        Ok(p)
    }

    fn compute_many(&self, mols: &[Molecule]) -> Result<Vec<ModelProperties>> {
        // 1. render input text with the template
        let mut txt = String::new();
        for mol in mols.iter() {
            let part = self.render_input(&mol)?;
            txt.push_str(&part);
        }

        // 2. call external engine
        info!("run in batch mode ...");
        let output = safe_call(&self.runfile, &txt)?;

        // 3. collect model properties
        let all = ModelProperties::parse_all(&output)?;

        Ok(all)
    }
}

/// Call external script
fn safe_call<P: AsRef<Path>>(runfile: P, input: &str) -> Result<String> {
    let runfile = runfile.as_ref();

    info!("run script file: {}", &runfile.display());

    let mut output = String::new();
    let tdir = new_scrdir()?;

    info!("scratch dir: {}", tdir.path().display());

    let cmdline = format!("{}", runfile.display());
    output = cmd!(&cmdline)
        .dir(tdir.path())
        .input(input)
        .read()
        .map_err(|e| {
            // keep temporary directory alive for debugging
            let path = tdir.into_path();
            error!("Job failed.\nPlease check scratch directory:\n {}", path.display());

            format_err!("failed to submit:\n {:?}: {:?}",
                        &runfile.display(),
                        e)
        })?;

    Ok(output)
}
// chemical model:1 ends here
