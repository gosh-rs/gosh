// docs
// command line tool for running a blackbox model

// features:
// - geometry optimization
// - collect optimization trajectory
// - output optimized structure




// imports

use gosh::cmd_utils::*;
use std::path::PathBuf;

use gchemol::Molecule;
use gosh::apps::optimization::lbfgs::lbfgs_opt;
use gosh::models::*;

// cmdline

/// An universal runner for Blackbox Model
#[derive(Debug, StructOpt)]
struct Cli {
    /// Input molecule file
    #[structopt(parse(from_os_str))]
    molfile: PathBuf,

    /// Compute many molecules in bundle.
    #[structopt(short = "b", long = "bundle")]
    bundle: bool,

    /// Dry-run mode: generate input file, but no real calculation.
    #[structopt(long = "dry-run")]
    dry: bool,

    /// Don't remove scratch files if calculation completed.
    #[structopt(long = "keep")]
    keep: bool,

    /// Optimize molecule using the builtin LBFGS optimizer.
    #[structopt(long = "opt")]
    opt: bool,

    /// Template directory with all related files. The default is current
    /// directory.
    #[structopt(short = "t", long = "bbm-dir", parse(from_os_str))]
    bbmdir: Option<PathBuf>,

    /// Output the caputured structure. e.g.: -o foo.xyz
    #[structopt(short = "o", long = "output", parse(from_os_str))]
    output: Option<PathBuf>,

    #[structopt(flatten)]
    verbosity: Verbosity,
}

fn main() -> CliResult {
    let args = Cli::from_args();
    args.verbosity.setup_env_logger(&env!("CARGO_PKG_NAME"))?;

    // 1. load molecules
    info!("input molecule file: {}", &args.molfile.display());
    let mols = gchemol::io::read(&args.molfile)?;
    info!("loaded {} molecules.", mols.len());

    // 2. construct the model
    let mut bbm = if let Some(ref d) = args.bbmdir {
        BlackBox::from_dir(&d)
    } else {
        BlackBox::from_dir(std::env::current_dir()?)
    };

    // 3. process molecules using the model
    let mut keep = args.keep;
    if let Err(e) = process_molecules(args, &mut bbm, mols) {
        error!("Job failed:\n {:?}", e);
        keep = true;
    }

    if keep {
        bbm.keep_scratch_files();
    }

    Ok(())
}

// process

fn process_molecules(args: Cli, mut bbm: &mut BlackBox, mols: Vec<Molecule>) -> Result<()> {
    let mut final_mols = vec![];
    let mut keep = args.keep;
    if !args.bundle {
        info!("run in normal mode ...");
        for mol in mols.iter() {
            // 3. call external engine
            if !args.dry {
                if args.opt {
                    println!("optimization with LBFGS");
                    let mut mol = mol.clone();
                    mol.recenter();
                    let mp = lbfgs_opt(&mol, bbm, 0.1)?;
                    println!("{:}", mp);
                    // collect molecules
                    if let Some(mut mol) = mp.molecule {
                        if let Some(energy) = mp.energy {
                            mol.name = format!("energy = {:-10.4}", energy);
                        }
                        final_mols.push(mol);
                    }
                } else {
                    let p = bbm.compute(&mol)?;
                    println!("{:}", p);
                    // collect molecules
                    if let Some(mut mol) = p.molecule {
                        if let Some(energy) = p.energy {
                            mol.name = format!("energy = {:-10.4}", energy);
                        }
                        final_mols.push(mol);
                    }
                }
            } else {
                println!("{:}", bbm.render_input(mol)?);
            }
        }
    } else {
        info!("run in bundle mode ...");
        if !args.dry {
            if args.opt {
                unimplemented!()
            } else {
                let all = bbm.compute_bundle(&mols)?;
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
            }
        } else {
            println!("{:}", bbm.render_input_bundle(&mols)?);
        }
    }

    info!("found {} molecules.", final_mols.len());
    // output molecules
    if let Some(path) = args.output {
        if final_mols.len() == 0 {
            error!("no molecules was collected!");
        } else {
            println!("file saved to: {:}", path.display());
            gchemol::io::write(path, &final_mols)?;
        }
    }

    Ok(())
}
