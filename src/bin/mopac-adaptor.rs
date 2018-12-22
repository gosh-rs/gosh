// mopac-adaptor.rs
// :PROPERTIES:
// :header-args: :comments org :tangle src/bin/mopac-adaptor.rs
// :END:

#[macro_use]
extern crate duct;

extern crate gchemol;
extern crate gosh;

use std::path::PathBuf;

use gchemol::{io, prelude::*, Molecule};
use gosh::cmd_utils::*;

use gosh::adaptors::*;
use gosh::models::*;

/// Read MOPAC calculated results, format them as standard external model results.
#[derive(Debug, StructOpt)]
struct Cli {
    /// MOPAC generated output file
    #[structopt(parse(from_os_str))]
    outfile: PathBuf,

    /// Parse all result entries found in the output
    #[structopt(short = "a", long = "all")]
    all: bool,

    #[structopt(flatten)]
    verbosity: Verbosity,
}

fn main() -> Result<()> {
    let args = Cli::from_args();
    args.verbosity.setup_env_logger(&env!("CARGO_PKG_NAME"))?;

    // 1. read mopac output
    let outfile = &args.outfile;

    let mopac = MOPAC();
    if args.all {
        for d in mopac.parse_all(&outfile)? {
            if d.is_empty() {
                bail!("ee");
            }
            println!("{:}", d);
        }
    } else {
        let d = mopac.parse_last(&outfile)?;
        if d.is_empty() {
            bail!("ee");
        }
        println!("{:}", d);
    }

    Ok(())
}
