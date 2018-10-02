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

pub struct RunnerOptions {
    pub runfile: PathBuf,
    pub tplfile: PathBuf,
}

impl Default for RunnerOptions {
    fn default() -> Self {
        RunnerOptions {
            runfile: "submit.sh".into(),
            tplfile: "input.hbs".into(),
        }
    }
}

impl RunnerOptions {
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

        let mut ropt = RunnerOptions::default();
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
