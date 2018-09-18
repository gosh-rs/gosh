// [[file:~/Workspace/Programming/gosh/gosh.note::*src][src:1]]
#[macro_use] extern crate duct;
#[macro_use] extern crate quicli;

extern crate gosh;
extern crate gchemol;

use std::path::{Path, PathBuf};
use quicli::prelude::*;

use gchemol::Molecule;
use gosh::models::dftb;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(help = "Input molecule file", parse(from_os_str))]
    molfile: PathBuf,
    #[structopt(short = "e", long = "script", help = "Executable script to submit job", default_value = "./submit.sh", parse(from_os_str))]
    runfile: PathBuf,
    #[structopt(flatten)]
    verbosity: Verbosity,
}

main!(|args: Cli, log_level: verbosity| {
    info!("input molecule file: {}", &args.molfile.display());
    let mol = Molecule::from_file(args.molfile)?;

    let mresults = dftb::run(&mol, args.runfile)?;
    println!("{:#?}", mresults.energy);
});
// src:1 ends here
