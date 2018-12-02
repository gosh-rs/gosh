// nebrun.rs
// :PROPERTIES:
// :header-args: :comments org :tangle src/bin/nebrun.rs
// :END:
// command line tool for NEB calculations


use std::path::PathBuf;
// fix quicli 2018 edition error
use quicli::prelude::structopt::StructOpt;
use quicli::prelude::*;
use quicli::main;

use gchemol::io;

use gosh::apps::{
    optimization::neb::NEB,
};
use gosh::models::*;

/// An universal runner for Blackbox Model
#[derive(Debug, StructOpt)]
struct Cli {
    /// Input molecule file
    #[structopt(parse(from_os_str))]
    molfile: PathBuf,

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
    let mut mols = io::read(args.molfile)?;
    info!("loaded {} molecules.", mols.len());

    let bbm = if let Some(d) = args.tpldir {
        BlackBox::from_dotenv(&d)
    } else {
        BlackBox::default()
    };

    let mut neb = NEB::new(mols);

    neb.run(&bbm)?;

    // output molecules
    if let Some(path) = args.output {
        let mut mols = vec![];
        for image in neb.images {
            mols.push(image.mol.clone());
        }

        println!("file saved to: {:}", path.display());
        io::write(path, &mols)?;
    }
});
