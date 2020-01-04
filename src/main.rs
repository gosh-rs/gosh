// imports

use gosh::cmd_utils::*;

use linefeed::{Interface, ReadResult};

use std::path::PathBuf;

use gosh::cli::*;

// gosh

#[derive(StructOpt, Debug)]
#[structopt(name = "gosh", about = "gosh")]
struct Gosh {
    #[structopt(flatten)]
    cmd: GoshCmd,
}

// REPL

fn get_history_file() -> Result<PathBuf> {
    match dirs::home_dir() {
        Some(path) => {
            let filename = path.join(".gosh.history");
            Ok(filename)
        }
        None => bail!("Impossible to get your home dir!"),
    }
}

fn start_gosh_cmd_loop() -> CliResult {
    let interface = Interface::new("rusty gosh")?;

    let version = env!("CARGO_PKG_VERSION");
    println!("This is the rusty gosh shell version {}.", version);
    println!("Enter \"help\" or \"?\" for a list of commands.");
    println!("Press Ctrl-D or enter \"quit\" or \"q\" to exit.");
    println!("");

    interface.set_prompt("gosh> ")?;
    interface.set_completer(std::sync::Arc::new(linefeed::complete::PathCompleter));

    let mut commander = Commander::new();

    let history_file = get_history_file().unwrap();
    if let Err(e) = interface.load_history(&history_file) {
        if e.kind() == std::io::ErrorKind::NotFound {
            println!(
                "History file {} doesn't exist, not loading history.",
                history_file.display()
            );
        } else {
            eprintln!(
                "Could not load history file {}: {}",
                history_file.display(),
                e
            );
        }
    }

    while let ReadResult::Input(line) = interface.read_line()? {
        let line = line.trim();
        if !line.is_empty() {
            interface.add_history(line.to_owned());

            let mut args: Vec<_> = line.split_whitespace().collect();
            args.insert(0, "gosh>");

            match GoshCmd::from_iter_safe(&args) {
                // show subcommands
                Ok(GoshCmd::Help {}) => {
                    let mut app = GoshCmd::clap();
                    app.print_help();
                    println!("");
                }

                Ok(GoshCmd::Quit {}) => {
                    if let Err(e) = interface.save_history(&history_file) {
                        eprintln!(
                            "Could not save history file {}: {}",
                            history_file.display(),
                            e
                        );
                    }

                    break;
                }

                // apply subcommand
                Ok(x) => {
                    if let Err(e) = commander.action(&x) {
                        eprintln!("{:?}", e);
                    }
                }

                // show subcommand usage
                Err(e) => {
                    println!("{}", e.message);
                }
            }
        } else {
            println!("");
        }
    }

    Ok(())
}

fn main() -> CliResult {
    let args: Vec<String> = std::env::args().collect();

    // entry shell mode or subcommands mode
    if args.len() > 1 {
        let args = Gosh::from_args();
        setup_logger();

        let mut commander = Commander::new();
        commander.action(&args.cmd);
    } else {
        start_gosh_cmd_loop();
    }

    Ok(())
}
