// bin/vasp-adaptor.rs
// :PROPERTIES:
// :header-args: :comments org :tangle src/bin/vasp-adaptor.rs
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

/// Read VASP calculated results, format them as standard external model results.
#[derive(Debug, StructOpt)]
struct Cli {
    /// VASP generated OUTCAR file
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

    // 1. read vasp output
    let outfile = &args.outfile;

    let vasp = Vasp();
    if args.all {
        for d in vasp.parse_all(&outfile)? {
            if d.is_empty() {
                bail!("ee");
            }
            println!("{:}", d);
        }
    } else {
        let d = vasp.parse_last(&outfile)?;
        if d.is_empty() {
            bail!("ee");
        }
        println!("{:}", d);
    }

    Ok(())
}
