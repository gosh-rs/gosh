// docs
// command line tool for running a blackbox model

// features:
// - geometry optimization
// - collect optimization trajectory
// - output optimized structure




// imports

use std::path::PathBuf;

use gosh::model::*;
use gosh_core::*;

use gchemol::Molecule;
use gut::prelude::*;

use gut::cli::*;
use structopt::*;
use vecfx::*;

// cmdline

/// An universal runner for Blackbox Model
#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(flatten)]
    verbose: gut::cli::Verbosity,

    /// Input molecule file
    #[structopt(parse(from_os_str))]
    molfile: PathBuf,

    /// Compute many molecules in bunch.
    #[structopt(short = "b", long = "bunch")]
    bunch: bool,

    /// Dry-run mode: generate input file, but no real calculation.
    #[structopt(long = "dry-run")]
    dry: bool,

    /// Don't remove scratch files if calculation completed.
    #[structopt(long)]
    keep: bool,

    /// Optimize molecule using the builtin LBFGS optimizer.
    #[structopt(long)]
    opt: bool,

    /// Forces convergence criterion for optimizing molecule geometry.
    #[structopt(long, default_value="0.1")]
    fmax: f64,

    /// Max allowed number of iterations during optimization.
    #[structopt(long, default_value="50")]
    nmax: usize,

    /// Template directory with all related files. The default is current
    /// directory.
    #[structopt(short = "t", long = "bbm-dir", parse(from_os_str))]
    bbmdir: Option<PathBuf>,

    /// Output the caputured structure. e.g.: -o foo.xyz
    #[structopt(short = "o", long = "output", parse(from_os_str))]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Cli::from_args();
    args.verbose.setup_logger();

    // 1. load molecules
    info!("input molecule file: {}", &args.molfile.display());
    let mols = gchemol::io::read_all(&args.molfile)?;
    info!("loaded {} molecules.", mols.len());

    // 2. construct the model
    let mut bbm = if let Some(ref d) = args.bbmdir {
        BlackBox::from_dir(&d)?
    } else {
        BlackBox::from_dir(std::env::current_dir()?)?
    };

    // 3. process molecules using the model
    let mut keep = args.keep;
    if let Err(e) = process_molecules(args, &mut bbm, mols) {
        error!("bbm failed:\n {:?}", e);
        keep = true;
    }

    if keep {
        bbm.keep_scratch_files();
    }

    Ok(())
}

// process

fn process_molecules(args: Cli, bbm: &mut BlackBox, mols: Vec<Molecule>) -> Result<()> {
    let mut final_mols = vec![];
    let mut keep = args.keep;

    if !args.bunch {
        info!("run in normal mode ...");
        for mol in mols.iter() {
            // 3. call external engine
            if !args.dry {
                if args.opt {
                    println!("Optimizing molecule in LBFGS algorithm ...");
                    let mut mol = mol.clone();
                    let optimized =
                        gosh_optim::Optimizer::new(args.nmax, args.fmax).optimize_geometry(&mut mol, bbm)?;
                    let mp = optimized.computed;
                    println!("{:}", mp);
                    // collect molecules
                    if let Some(mol) = mp.get_molecule() {
                        let mut mol = mol.clone();
                        if let Some(energy) = mp.get_energy() {
                            let name = format!("energy = {:-10.4}", energy);
                            mol.set_title(&name);
                        }
                        final_mols.push(mol);
                    }
                } else {
                    let p = bbm.compute(&mol)?;
                    println!("{:}", p);
                    // collect molecules
                    if let Some(mol) = p.get_molecule() {
                        let mut mol = mol.clone();
                        if let Some(energy) = p.get_energy() {
                            let name = format!("energy = {:-10.4}", energy);
                            mol.set_title(&name);
                        }
                        final_mols.push(mol);
                    }
                }
            } else {
                println!("{:}", bbm.render_input(mol)?);
            }
        }
    } else {
        info!("run in bunch mode ...");
        if !args.dry {
            if args.opt {
                unimplemented!()
            } else {
                let all = bbm.compute_bunch(&mols)?;
                for p in all {
                    println!("{:}", p);
                    // collect molecules
                    if let Some(mol) = p.get_molecule() {
                        let mut mol = mol.clone();
                        // save energy as comment
                        if let Some(energy) = p.get_energy() {
                            let name = format!("energy = {:-10.4}", energy);
                            mol.set_title(&name);
                        }
                        final_mols.push(mol);
                    }
                }
            }
        } else {
            println!("{:}", bbm.render_input_bunch(&mols)?);
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
