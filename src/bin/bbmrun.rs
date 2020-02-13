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
}

fn main() -> Result<()> {
    let args = Cli::from_args();
    setup_logger();

    // 1. load molecules
    info!("input molecule file: {}", &args.molfile.display());
    let mols = gchemol::io::read_all(&args.molfile)?;
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
        error!("bbm failed:\n {:?}", e);
        keep = true;
    }

    if keep {
        bbm.keep_scratch_files();
    }

    Ok(())
}

// lbfgs

use ::liblbfgs::lbfgs;

/// Optimize molecule using blackbox model
/// # Parameters
/// - mol: target molecule
/// - model: chemical model for properties evaluation
/// - fmax: the max force for convergence
pub(self) fn lbfgs_opt<T: ChemicalModel>(
    mol: &Molecule,
    model: &mut T,
    fmax: f64,
) -> Result<ModelProperties> {
    let mp = model.compute(&mol)?;
    if let Some(energy) = mp.get_energy() {
        println!("current energy = {:-10.4}", energy);
    } else {
        bail!("no energy")
    }

    let mut mol = mol.clone();
    let mut positions: Vec<_> = mol.positions().collect();
    let mut arr_x = positions.as_mut_flat();
    let mut icall = 0;
    lbfgs()
        .with_initial_step_size(1.0 / 75.0)
        .with_max_step_size(0.4)
        .with_damping(true)
        .with_max_linesearch(5)
        .with_linesearch_gtol(0.999)
        .with_epsilon(0.01)
        .minimize(
            &mut arr_x,
            |arr_x, gx| {
                let positions = arr_x.as_3d().to_vec();
                mol.set_positions(positions);
                let mp = model.compute(&mol)?;

                // set gradients
                if let Some(forces) = &mp.get_forces() {
                    let forces = forces.as_flat();
                    assert_eq!(gx.len(), forces.len());
                    for i in 0..forces.len() {
                        gx[i] = -forces[i];
                    }
                } else {
                    bail!("no forces!");
                }

                let fx = if let Some(energy) = mp.get_energy() {
                    energy
                } else {
                    bail!("no energy!");
                };

                let fnorms: Vec<_> = gx.as_3d().iter().map(|v| v.vec2norm()).collect();
                let fm = fnorms.max();
                println!(
                    "niter = {:5} energy = {:-10.4} fmax = {:-10.4}",
                    icall, fx, fm
                );
                icall += 1;

                Ok(fx)
            },
            |prgr| {
                let fnorms: Vec<_> = prgr.gx.chunks(3).map(|v| v.vec2norm()).collect();
                let fcur = fnorms.max();
                false
            },
        )?;

    let mp = model.compute(&mol)?;
    Ok(mp)
}

// process

fn process_molecules(args: Cli, mut bbm: &mut BlackBox, mols: Vec<Molecule>) -> Result<()> {
    let mut final_mols = vec![];
    let mut keep = args.keep;

    if !args.bunch {
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
