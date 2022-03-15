// [[file:../gosh.note::7d1be705][7d1be705]]
use super::*;
use crate::model::*;

use gchemol::Molecule;
use gosh_core::*;
use vecfx::*;
// 7d1be705 ends here

// [[file:../gosh.note::9497e7ed][9497e7ed]]
use gut::cli::*;

/// An universal runner for Blackbox Model
#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(flatten)]
    verbose: Verbosity,

    /// Input molecule file
    #[structopt(parse(from_os_str))]
    molfile: PathBuf,

    /// Compute many molecules in bunch.
    #[structopt(short = 'b', long = "bunch")]
    bunch: bool,

    /// Dry-run mode: generate input file, but no real calculation.
    #[structopt(long = "dry-run")]
    dry: bool,

    /// Don't remove scratch files if calculation completed.
    #[structopt(long)]
    keep: bool,

    /// Optimize molecule using the builtin LBFGS optimizer.
    #[structopt(long, conflicts_with = "bunch")]
    opt: bool,

    /// Forces convergence criterion for optimizing molecule geometry.
    #[structopt(long, default_value = "0.1")]
    fmax: f64,

    /// Max allowed number of iterations during optimization.
    #[structopt(long, default_value = "50")]
    nmax: usize,

    /// Template directory with all related files. The default is current
    /// directory.
    #[structopt(short = 't', long = "bbm-dir", parse(from_os_str))]
    bbmdir: Option<PathBuf>,

    /// Output the caputured structure. e.g.: -o foo.xyz
    #[structopt(short = 'o', long = "output", parse(from_os_str))]
    output: Option<PathBuf>,

    #[structopt(flatten)]
    checkpoint: gosh_database::CheckpointDb,
}
// 9497e7ed ends here

// [[file:../gosh.note::a3e4479e][a3e4479e]]
/// Extract final molecule from calculated model properties
fn extract_mol_from(mp: &ModelProperties) -> Option<Molecule> {
    let mut mol = mp.get_molecule()?.clone();
    if let Some(energy) = mp.get_energy() {
        // save energy as comment (useful for .xyz file)
        let name = format!("energy = {:-10.4}", energy);
        mol.set_title(&name);
    }

    mol.into()
}

/// Compute a list of molecules
fn compute_mps(bbm: &mut BlackBoxModel, mols: Vec<Molecule>, bunch_mode: bool) -> Result<Vec<ModelProperties>> {
    if bunch_mode {
        bbm.compute_bunch(&mols)
    } else {
        mols.iter().map(|mol| bbm.compute(mol)).collect()
    }
}

fn compute(bbm: &mut BlackBoxModel, mols: Vec<Molecule>, bunch_mode: bool) -> Result<Vec<Molecule>> {
    compute_mps(bbm, mols, bunch_mode)?
        .into_iter()
        .inspect(|mp| println!("{}", mp))
        .map(|mp| extract_mol_from(&mp).ok_or(format_err!("no mol in model properties")))
        .collect()
}

fn dry_run(bbm: &mut BlackBoxModel, mols: Vec<Molecule>, bunch_mode: bool) -> Result<()> {
    if bunch_mode {
        println!("{:}", bbm.render_input_bunch(&mols)?);
    } else {
        for mol in mols.iter() {
            println!("{:}", bbm.render_input(mol)?);
        }
    }

    Ok(())
}
// a3e4479e ends here

// [[file:../gosh.note::*process][process:1]]
fn process_molecules(args: Cli, bbm: &mut BlackBoxModel, mols: Vec<Molecule>) -> Result<()> {
    if args.dry {
        dry_run(bbm, mols, args.bunch)?;
        return Ok(());
    }

    let final_mols = if !args.bunch {
        info!("run in normal mode ...");
        let mut final_mols = vec![];
        if args.opt {
            for mol in mols.iter() {
                println!("Optimizing molecule using LBFGS algorithm ...");
                let mut mol = mol.clone();

                let optimized = gosh_optim::Optimizer::new(args.fmax, args.nmax)
                    .checkpoint(args.checkpoint.create())
                    .optimize_geometry(&mut mol, bbm)?;

                let mp = optimized.computed;
                println!("{:}", mp);
                if let Some(mol) = extract_mol_from(&mp) {
                    final_mols.push(mol);
                } else {
                    bail!("no collected mol in mp: {:?}", mp);
                }
            }
            final_mols
        } else {
            compute(bbm, mols, false)?
        }
    } else {
        info!("run in bunch mode ...");
        compute(bbm, mols, true)?
    };

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
// process:1 ends here

// [[file:../gosh.note::*main][main:1]]
pub fn bbm_enter_main() -> Result<()> {
    let args = Cli::from_args();
    args.verbose.setup_logger();

    // 1. load molecules
    info!("input molecule file: {}", &args.molfile.display());
    let mols = gchemol::io::read_all(&args.molfile)?;
    info!("loaded {} molecules.", mols.len());

    // 2. construct the model
    let mut bbm = if let Some(ref d) = args.bbmdir {
        BlackBoxModel::from_dir(&d)?
    } else {
        BlackBoxModel::from_dir(std::env::current_dir()?)?
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
// main:1 ends here
