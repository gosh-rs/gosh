// [[file:../gosh.note::*imports][imports:1]]
use crate::common::*;

use crate::cli::Commander;
use crate::cli::GoshCmd;
use structopt::StructOpt;

use rustyline::error::ReadlineError;
use rustyline::Editor;
// imports:1 ends here

// [[file:../gosh.note::*constants][constants:1]]
const PROMPT: &str = "gosh> ";

// Return the default history file: ~/.gosh.history or .gosh.history
fn get_history_file() -> PathBuf {
    dirs::home_dir().unwrap_or_default().join(".gosh.history")
}
// constants:1 ends here

// [[file:../gosh.note::*core][core:1]]
pub struct Interpreter {
    history_file: PathBuf,
    editor: Editor<helper::MyHelper>,
    commander: Commander,
}
impl Interpreter {
    pub fn new() -> Self {
        Self {
            history_file: get_history_file(),
            editor: create_readline_editor(),
            commander: Commander::new(),
        }
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "gosh", about = "gosh")]
struct Gosh {
    #[structopt(flatten)]
    cmd: GoshCmd,
}

fn create_readline_editor() -> Editor<helper::MyHelper> {
    use rustyline::{ColorMode, CompletionType, Config, Editor};

    let config = Config::builder()
        .color_mode(rustyline::ColorMode::Enabled)
        .history_ignore_space(true)
        .completion_type(CompletionType::Fuzzy)
        .max_history_size(1000)
        .build();

    let mut rl = Editor::with_config(config);
    let h = helper::MyHelper::new();
    rl.set_helper(Some(h));
    rl
}
// core:1 ends here

// [[file:../gosh.note::*repl][repl:1]]
impl Interpreter {
    fn read_eval_print(&mut self) -> Result<()> {
        let line = self.editor.readline(PROMPT)?;
        let line = line.trim();
        if !line.is_empty() {
            self.editor.add_history_entry(line);
        }

        let mut args: Vec<_> = line.split_whitespace().collect();
        args.insert(0, PROMPT);

        match GoshCmd::from_iter_safe(&args) {
            // show subcommands
            Ok(GoshCmd::Help {}) => {
                let mut app = GoshCmd::clap();
                app.print_help();
                println!("");
            }
            // apply subcommand
            Ok(x) => {
                if let Err(e) = self.commander.action(&x) {
                    eprintln!("{:?}", e);
                }
            }
            Ok(GoshCmd::Quit {}) => bail!("Quit"),
            // show subcommand usage
            Err(e) => println!("{}", e.message),
        }

        Ok(())
    }

    fn load_history(&mut self) -> Result<()> {
        self.editor.load_history(&self.history_file).context("no history")?;
        Ok(())
    }

    fn save_history(&mut self) -> Result<()> {
        self.editor
            .save_history(&self.history_file)
            .context("write gosh history file")?;
        Ok(())
    }

    pub fn start_repl(&mut self) -> Result<()> {
        let version = env!("CARGO_PKG_VERSION");
        println!("This is the rusty gosh shell version {}.", version);
        println!("Enter \"help\" or \"?\" for a list of commands.");
        println!("Press Ctrl-D or enter \"quit\" or \"q\" to exit.");
        println!("");

        let _ = self.load_history();
        loop {
            self.read_eval_print()?;
        }
        self.save_history()?;

        Ok(())
    }
}
// repl:1 ends here

// [[file:../gosh.note::*scripting][scripting:1]]
impl Interpreter {
    pub fn interpret_script(&mut self, script: &str) -> Result<()> {
        todo!()
    }
}
// scripting:1 ends here

// [[file:../gosh.note::*helper][helper:1]]
mod helper {
    use super::*;

    use rustyline::completion::{FilenameCompleter, Pair};
    use rustyline::error::ReadlineError;
    use rustyline::Context;
    use rustyline_derive::{Completer, Helper, Highlighter, Validator};

    #[derive(Helper, Highlighter, Validator)]
    pub struct MyHelper {
        completer: FilenameCompleter,
        colored_prompt: String,
    }

    impl rustyline::completion::Completer for MyHelper {
        type Candidate = Pair;

        fn complete(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Result<(usize, Vec<Pair>), ReadlineError> {
            self.completer.complete(line, pos, ctx)
        }
    }

    impl MyHelper {
        pub fn new() -> Self {
            Self {
                completer: FilenameCompleter::new(),
                colored_prompt: "".to_owned(),
            }
        }
    }

    // FIXME: cannot be derived using rustyline_derive
    impl rustyline::hint::Hinter for MyHelper {
        type Hint = String;

        fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
            None
        }
    }
}
// helper:1 ends here

// [[file:../gosh.note::*pub][pub:1]]
pub fn repl_enter_main() -> Result<()> {
    use gut::cli::*;

    let args: Vec<String> = std::env::args().collect();

    // entry shell mode or subcommands mode
    if args.len() > 1 {
        let args = Gosh::from_args();
        setup_logger();

        let mut commander = Commander::new();
        commander.action(&args.cmd)?;
    } else {
        Interpreter::new().start_repl()?;
    }

    Ok(())
}
// pub:1 ends here
