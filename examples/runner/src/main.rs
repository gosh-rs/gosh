// main

// [[file:~/Workspace/Programming/gosh/gosh.note::*main][main:1]]
#[macro_use] extern crate duct;
#[macro_use] extern crate quicli;

extern crate gosh;
extern crate gchemol;

use std::path::{Path, PathBuf};

use quicli::prelude::*;
use ::structopt::StructOpt;

use gchemol::{
    io,
    Molecule,
    prelude::*,
};

use gosh::models::*;

/// A universal runner for Blackbox Model
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

use runner::RunnerOptions;

main!(|args: Cli, log_level: verbosity| {
    // 1. load molecules
    info!("input molecule file: {}", &args.molfile.display());
    let mols = io::read(args.molfile)?;
    info!("loaded {} molecules.", mols.len());

    let ropts = if let Some(d) = args.tpldir {
        RunnerOptions::from_dotenv(&d)
    } else {
        RunnerOptions::default()
    };

    // 2. load input template
    let template = io::read_file(ropts.tplfile)
        .map_err(|e| format_err!("failed to load template:\n {}", e))?;

    let mut final_mols = vec![];
    if ! args.join {
        info!("run in normal mode ...");
        for mol in mols.iter() {
            let txt = mol.render_with(&template)?;
            // 3. call external engine
            if ! args.dry {
                let output = safe_call(&ropts.runfile, &txt)?;
                let p: ModelProperties = output.parse()?;
                println!("{:}", p);

                // collect molecules
                if let Some(mut mol) = p.molecule {
                    if let Some(energy) = p.energy {
                        mol.name = format!("energy = {:-10.4}", energy);
                    }
                    final_mols.push(mol);
                }
            } else {
                println!("{:}", txt);
            }
        }
    } else {
        info!("run in batch mode ...");
        let mut txt = String::new();
        for mol in mols.iter() {
            let part = mol.render_with(&template)?;
            txt.push_str(&part);
        }
        // 3. call external engine
        if ! args.dry {
            let output = safe_call(&ropts.runfile, &txt)?;
            let all = ModelProperties::parse_all(&output)?;
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
            println!("{:}", txt);
        }
    }

    info!("found {} molecules.", final_mols.len());

    // output molecules
    if let Some(path) = args.output {
        println!("file saved to: {:}", path.display());
        io::write(path, &final_mols)?;
    }
});

/// Call external script
fn safe_call<P: AsRef<Path>>(runfile: P, input: &str) -> Result<String> {
    use runner::new_scrdir;

    let runfile = runfile.as_ref();

    info!("run script file: {}", &runfile.display());

    let mut output = String::new();
    let tdir = new_scrdir()?;

    info!("scratch dir: {}", tdir.path().display());

    let cmdline = format!("{}", runfile.display());
    output = cmd!(&cmdline)
        .dir(tdir.path())
        .input(input)
        .read()
        .map_err(|e| format_err!("failed to submit:\n {:?}: {:?}",
                                 &runfile.display(),
                                 e)
        )?;

    Ok(output)
}
// main:1 ends here
