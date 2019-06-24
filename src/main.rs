// imports

use gosh::cmd_utils::*;

use linefeed::{Interface, ReadResult};

use dirs;
use std::path::PathBuf;

use gosh::cli::*;

// gosh

#[derive(StructOpt, Debug)]
#[structopt(name = "gosh", about = "gosh")]
struct Gosh {
    #[structopt(flatten)]
    verbosity: Verbosity,

    #[structopt(flatten)]
    cmd: GoshCmd,
}

// cmd loop

fn start_gosh_cmd_loop() -> CliResult {
    let interface = Interface::new("rusty gosh")?;

    println!(
        "This is the rusty gosh shell version {}.",
        env!("CARGO_PKG_VERSION")
    );
    println!("Enter \"help\" or \"?\" for a list of commands.");
    println!("Press Ctrl-D or enter \"quit\" or \"q\" to exit.");
    println!("");

    interface.set_prompt("gosh> ")?;

    let mut commander = Commander::new();
    while let ReadResult::Input(line) = interface.read_line()? {
        println!("");
        let line = line.trim();
        if !line.is_empty() {
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
                    break;
                }

                // apply subcommand
                Ok(x) => {
                    commander.action(&x);
                }

                // show subcommand usage
                Err(e) => {
                    println!("{}", e.message);
                }
            }
        }
    }

    Ok(())
}

fn main() -> CliResult {
    let args: Vec<String> = std::env::args().collect();

    // entry shell mode or subcommands mode
    if args.len() > 1 {
        let args = Gosh::from_args();
        args.verbosity.setup_env_logger(&env!("CARGO_PKG_NAME"))?;

        let mut commander = Commander::new();
        commander.action(&args.cmd);
    } else {
        start_gosh_cmd_loop();
    }

    Ok(())
}
