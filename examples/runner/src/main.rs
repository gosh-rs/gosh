// src

#[macro_use] extern crate duct;
#[macro_use] extern crate quicli;

extern crate gosh;
extern crate gchemol;

use std::path::PathBuf;
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

    /// Join multiple molecules into a single input
    #[structopt(short = "j", long = "join")]
    join: bool,

    /// Dry-run mode: generate input file, but no real calculation.
    #[structopt(short = "d", long = "dry")]
    dry: bool,

    #[structopt(flatten)]
    verbosity: Verbosity,
}

main!(|args: Cli, log_level: verbosity| {
    // 1. load molecules
    info!("input molecule file: {}", &args.molfile.display());
    let mols = io::read(args.molfile)?;
    info!("loaded {} molecules.", mols.len());

    // 2. load input template
    let template = io::read_file(args.tplfile)
        .map_err(|e| format_err!("failed to load template"))?;

    if ! args.join {
        info!("run in normal mode ...");
        for mol in mols.iter() {
            let txt = mol.render_with(&template)?;
            // 3. call external engine
            let output = safe_call(&args.runfile, &txt, args.dry)?;
            let x: ModelProperties = output.parse()?;
            println!("{:}", x);
        }
    } else {
        info!("run in batch mode ...");
        let mut txt = String::new();
        for mol in mols.iter() {
            let part = mol.render_with(&template)?;
            txt.push_str(&part);
        }
        // 3. call external engine
        let output = safe_call(&args.runfile, &txt, args.dry)?;
        let all = ModelProperties::parse_all(&output)?;
        println!("got {:#?} parts", all.len());
    }
});

/// Call external script
fn safe_call(runfile: &PathBuf, input: &str, dry: bool) -> Result<String> {
    info!("run script file: {}", &runfile.display());

    let mut output = String::new();
    if ! dry {
        // goto script parent directory
        let d = &runfile.parent().expect("failed to get run script's parent dir!");
        let cmdline = format!("{}", runfile.display());
        output = cmd!(&cmdline)
            .dir(d)
            .input(input)
            .read()
            .map_err(|e| format_err!("{:?}: {:?}",
                                     &runfile.display(),
                                     e)
            )?;
    } else {
        info!("dry run mode");
    }

    Ok(output)
}
