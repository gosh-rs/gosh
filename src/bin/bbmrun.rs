// bin/bbmrun.rs
// :PROPERTIES:
// :header-args: :comments org :tangle src/bin/bbmrun.rs
// :END:
// command line tool for run a blackbox model

#[macro_use] extern crate duct;
#[macro_use] extern crate quicli;

use std::path::{Path, PathBuf};
use quicli::prelude::*;
use quicli::main;
use ::structopt::StructOpt;

use gchemol::{
    io,
    Molecule,
    prelude::*,
};

use gosh::{
    models::*,
};

/// An universal runner for Blackbox Model
#[derive(Debug, StructOpt)]
struct Cli {
    /// Input molecule file
    #[structopt(parse(from_os_str))]
    molfile: PathBuf,

    /// Join multiple molecules into a single input
    #[structopt(short="j", long="join")]
    join: bool,

    /// Dry-run mode: generate input file, but no real calculation.
    #[structopt(long="dry-run")]
    dry: bool,

    /// Template directory with all related files. The default is current directory.
    #[structopt(short="t", long="template-dir", parse(from_os_str))]
    tpldir: Option<PathBuf>,

    /// Output the caputured structure. e.g.: -o foo.xyz
    #[structopt(short="o", long="output", parse(from_os_str))]
    output: Option<PathBuf>,

    #[structopt(flatten)]
    verbosity: Verbosity,
}

main!(|args: Cli, log_level: verbosity| {
    // 1. load molecules
    info!("input molecule file: {}", &args.molfile.display());
    let mols = io::read(args.molfile)?;
    info!("loaded {} molecules.", mols.len());

    let bbm = if let Some(d) = args.tpldir {
        BlackBox::from_dotenv(&d)
    } else {
        BlackBox::default()
    };

    let mut final_mols = vec![];
    if ! args.join {
        info!("run in normal mode ...");
        for mol in mols.iter() {
            // 3. call external engine
            if ! args.dry {
                let p = bbm.compute(&mol)?;
                println!("{:}", p);

                // collect molecules
                if let Some(mut mol) = p.molecule {
                    if let Some(energy) = p.energy {
                        mol.name = format!("energy = {:-10.4}", energy);
                    }
                    final_mols.push(mol);
                }
            } else {
                println!("{:}", bbm.render_input(mol)?);
            }
        }
    } else {
        info!("run in batch mode ...");
        if ! args.dry {
            let all = bbm.compute_many(&mols)?;
            for p in all {
                println!("{:}", p);
                // collect molecules
                if let Some(mut mol) = p.molecule {
                    // save energy as comment
                    if let Some(energy) = p.energy {
                        mol.name = format!("energy = {:-10.4}", energy);
                    }
                    final_mols.push(mol);
                }
            }
        }  else {
            for mol in mols.iter() {
                println!("{:}", bbm.render_input(mol)?);
            }
        }
    }

    info!("found {} molecules.", final_mols.len());

    // output molecules
    if let Some(path) = args.output {
        println!("file saved to: {:}", path.display());
        io::write(path, &final_mols)?;
    }
});
