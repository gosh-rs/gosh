// imports

use gut::cli::*;
use gut::prelude::*;
use std::path::PathBuf;
use structopt::*;

use gosh::cli::*;
use gosh_core::*;

// gosh

#[derive(StructOpt, Debug)]
#[structopt(name = "gosh", about = "gosh")]
struct Gosh {
    #[structopt(flatten)]
    cmd: GoshCmd,
}

fn get_history_file() -> Result<PathBuf> {
    match dirs::home_dir() {
        Some(path) => {
            let filename = path.join(".gosh.history");
            Ok(filename)
        }
        None => bail!("Impossible to get your home dir!"),
    }
}

// REPL/rustyline

fn start_gosh_cmd_loop() -> Result<()> {
    use rustyline::error::ReadlineError;
    use rustyline::Editor;

    let version = env!("CARGO_PKG_VERSION");
    println!("This is the rusty gosh shell version {}.", version);
    println!("Enter \"help\" or \"?\" for a list of commands.");
    println!("Press Ctrl-D or enter \"quit\" or \"q\" to exit.");
    println!("");

    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();

    // load history
    let history_file = get_history_file()?;
    if rl.load_history(&history_file).is_err() {
        println!("No previous history.");
    }

    let mut commander = Commander::new();
    loop {
        let readline = rl.readline("gosh> ");
        match readline {
            Ok(line) => {
                let line = line.trim();
                if !line.is_empty() {
                    rl.add_history_entry(line);
                }

                let mut args: Vec<_> = line.split_whitespace().collect();
                args.insert(0, "gosh>");
                match GoshCmd::from_iter_safe(&args) {
                    // show subcommands
                    Ok(GoshCmd::Help {}) => {
                        let mut app = GoshCmd::clap();
                        app.print_help();
                        println!("");
                    }
                    // apply subcommand
                    Ok(x) => {
                        if let Err(e) = commander.action(&x) {
                            eprintln!("{:?}", e);
                        }
                    }
                    Ok(GoshCmd::Quit {}) => break,
                    // show subcommand usage
                    Err(e) => println!("{}", e.message),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history(&history_file).context("write gosh history file")?;

    Ok(())
}

// main

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // entry shell mode or subcommands mode
    if args.len() > 1 {
        let args = Gosh::from_args();
        setup_logger();

        let mut commander = Commander::new();
        commander.action(&args.cmd)?;
    } else {
        start_gosh_cmd_loop();
    }

    Ok(())
}
