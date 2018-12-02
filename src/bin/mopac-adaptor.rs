// mopac-adaptor.rs
// :PROPERTIES:
// :header-args: :comments org :tangle src/bin/mopac-adaptor.rs
// :END:

#[macro_use] extern crate duct;

extern crate gosh;
extern crate gchemol;

use std::path::PathBuf;
// fix quicli 2018 edition error
use quicli::prelude::structopt::StructOpt;
use quicli::prelude::*;
use quicli::main;

use gchemol::{
    io,
    Molecule,
    prelude::*,
};

use gosh::models::*;
use gosh::adaptors::*;

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

main!(|args: Cli, log_level: verbosity| {
    // 1. read mopac output
    let outfile = &args.outfile;

    let mopac = MOPAC();
    if args.all {
        for d in mopac.parse_all(&outfile)? {
            if d.is_empty() {
                bail!("ee")
            }
            println!("{:}", d);
        }
    } else {
        let d = mopac.parse_last(&outfile)?;
        if d.is_empty() {
            bail!("ee")
        }
        println!("{:}", d);
    }
});
