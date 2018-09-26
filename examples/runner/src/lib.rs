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
            Ok(tempdir()?)
        }
    }
}
// scratch:1 ends here

// dotenv
// 从当前目录里读入相关的环境变量.

// 输入模板:
// : BBM_TPL_FILE=input.hbs
// 其中input.hbs是相当于模板所在目录而言的. 也可以指定绝对路径:
// : BBM_TPL_FILE=/abs/path/to/input.hbs

// 提交任务:
// : BBM_RUN_FILE=submit.sh

// 临时目录(这是相对于当前目录而言的):
// : BBM_SCR_DIR=/scratch


// [[file:~/Workspace/Programming/gosh/gosh.note::*dotenv][dotenv:1]]
use dotenv::dotenv;
use std::env;
use std::path::{Path, PathBuf};
use quicli::prelude::*;

/// Enter directory with environment variables from .env file
fn enter_dir_with_env(dir: &Path) -> Result<()>{
    info!("read dotenv vars from {}", dir.display());

    // change to directory
    env::set_current_dir(&dir)?;

    // read environment variables
    dotenv().ok();

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
