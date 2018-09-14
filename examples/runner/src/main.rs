// [[file:~/Workspace/Programming/gosh/gosh.note::*src][src:1]]
#[macro_use] extern crate duct;
#[macro_use] extern crate quicli;

extern crate gosh;
extern crate gchemol;

use std::path::{Path, PathBuf};
use quicli::prelude::*;

use gchemol::{
    io,
    Molecule,
    prelude::*,
};

use gosh::models::*;

#[derive(Debug, StructOpt)]
struct Cli {
    /// Input molecule file
    #[structopt(parse(from_os_str))]
    molfile: PathBuf,

    /// Executable script to submit job
    #[structopt(short = "e", long = "script", default_value = "./submit.sh", parse(from_os_str))]
    runfile: PathBuf,

    /// Template file for rendering molecule
    #[structopt(short = "t", long = "template", default_value = "./input.hbs", parse(from_os_str))]
    tplfile: PathBuf,

    #[structopt(flatten)]
    verbosity: Verbosity,
}

main!(|args: Cli, log_level: verbosity| {
    info!("input molecule file: {}", &args.molfile.display());
    let mol = Molecule::from_file(args.molfile)?;

    let template = io::read_file(args.tplfile)
        .map_err(|e| format_err!("failed to load template"))?;
    let txt = mol.render_with(&template)?;

    let runfile = &args.runfile;
    info!("run script file: {}", &runfile.display());

    // goto script parent directory
    let d = &runfile.parent().expect("failed to get run script's parent dir!");
    let cmdline = format!("{}", runfile.display());
    let output = cmd!(&cmdline)
        .dir(d)
        .input(txt)
        .read()
        .map_err(|e| format_err!("{:?}: {:?}",
                                 &runfile.display(),
                                 e)
        )?;

    let x: ModelResults = output.parse()?;
    println!("{:#?}", x);
});
// src:1 ends here
