// [[file:../gosh.note::e3304fe2][e3304fe2]]
use super::*;
use crate::cli::Commander;
// e3304fe2 ends here

// [[file:../gosh.note::af47268b][af47268b]]
use crate::cli::GoshCmd as Cmd;

struct Action {
    commander: Commander,
}

impl Action {
    pub fn new() -> Self {
        Self {
            commander: Commander::new(),
        }
    }
}

impl Actionable for Action {
    type Command = Cmd;

    /// Take action on REPL commands. Return Ok(true) will exit shell
    /// loop.
    fn act_on(&mut self, cmd: &Cmd) -> Result<bool> {
        match cmd {
            Cmd::Quit {} => return Ok(true),

            Cmd::Help {} => {
                let mut app = Cmd::command();
                if let Err(err) = app.print_help() {
                    eprintln!("clap error: {err:?}");
                }
                println!("");
            }

            o => {
                if let Err(e) = self.commander.action(&o) {
                    eprintln!("{:?}", e);
                }
            }
        }

        Ok(false)
    }
}
// af47268b ends here

// [[file:../gosh.note::4651ecd4][4651ecd4]]
use gosh_repl::{Actionable, Interpreter};

use super::*;
use gut::cli::*;
use gut::prelude::*;

#[derive(Parser, Debug)]
struct GoshCli {
    /// Execute gosh script
    #[clap(short = 'x')]
    script_file: Option<PathBuf>,

    #[clap(flatten)]
    verbose: Verbosity,
}

impl GoshCli {
    pub fn enter_main() -> Result<()> {
        let args: Vec<String> = std::env::args().collect();

        let action = Action::new();
        // enter shell mode or subcommands mode
        if args.len() > 1 {
            let args = Self::parse();
            args.verbose.setup_logger();

            if let Some(script_file) = &args.script_file {
                info!("Execute script file: {:?}", script_file);
                Interpreter::new(action).interpret_script_file(script_file)?;
            } else {
                info!("Reading batch script from stdin ..");
                let mut buffer = String::new();
                std::io::stdin().read_to_string(&mut buffer)?;
                Interpreter::new(action).interpret_script(&buffer)?;
            }
        } else {
            Interpreter::new(action).with_prompt("gosh> ").run()?;
        }

        Ok(())
    }
}

pub fn repl_enter_main() -> Result<()> {
    GoshCli::enter_main()?;
    Ok(())
}
// 4651ecd4 ends here
