// [[file:../gosh.note::*imports][imports:1]]
use crate::cli::Commander;
use crate::cli::GoshCmd;
use crate::common::*;

use clap::Clap;
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
struct Interpreter {
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

fn create_readline_editor() -> Editor<helper::MyHelper> {
    use rustyline::{ColorMode, CompletionType, Config, Editor};

    let config = Config::builder()
        .color_mode(rustyline::ColorMode::Enabled)
        .history_ignore_dups(true)
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
    fn continue_read_eval_print(&mut self) -> bool {
        match self.editor.readline(PROMPT) {
            Err(rustyline::error::ReadlineError::Eof) => false,
            Ok(line) => {
                let line = line.trim();
                if !line.is_empty() {
                    self.editor.add_history_entry(line);
                    self.continue_interpret_line(&line)
                } else {
                    true
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                false
            }
        }
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
        while self.continue_read_eval_print() {
            trace!("excuted one loop");
        }
        self.save_history()?;

        Ok(())
    }
}
// repl:1 ends here

// [[file:../gosh.note::*scripting][scripting:1]]
impl Interpreter {
    /// Interpret one line.
    fn continue_interpret_line(&mut self, line: &str) -> bool {
        use clap::IntoApp;

        if let Some(mut args) = shlex::split(line) {
            assert!(args.len() >= 1);
            args.insert(0, "gosh".into());
            match GoshCmd::try_parse_from(&args) {
                // show subcommands
                Ok(GoshCmd::Help {}) => {
                    let mut app = GoshCmd::into_app();
                    app.print_help();
                    println!("");
                }
                // handle quit command first
                Ok(GoshCmd::Quit {}) => return false,
                // apply subcommand
                Ok(x) => {
                    if let Err(e) = self.commander.action(&x) {
                        eprintln!("{:?}", e);
                    }
                }
                // show subcommand usage
                Err(e) => println!("{:}", e),
            }
            true
        } else {
            dbg!(line);
            false
        }
    }

    fn interpret_script(&mut self, script: &str) -> Result<()> {
        let lines = script.lines().filter(|s| !s.trim().is_empty());
        for line in lines {
            debug!("Execute: {:?}", line);
            if !self.continue_interpret_line(&line) {
                break;
            }
        }

        Ok(())
    }

    pub fn interpret_script_file(&mut self, script_file: &Path) -> Result<()> {
        let s = gut::fs::read_file(script_file)?;
        self.interpret_script(&s)?;
        Ok(())
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
            if suitable_for_path_complete(line) {
                self.completer.complete(line, pos, ctx)
            } else {
                let commands = get_subcommands();
                let pairs = commands
                    .into_iter()
                    .filter_map(|x| {
                        if x.starts_with(line) {
                            new_candidate(&x).into()
                        } else {
                            None
                        }
                    })
                    .collect();
                Ok((0, pairs))
            }
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

    fn new_candidate(x: &str) -> Pair {
        Pair {
            display: x.into(),
            replacement: x.into(),
        }
    }

    fn get_subcommands() -> Vec<String> {
        use clap::IntoApp;

        let app = GoshCmd::into_app();
        app.get_subcommands().map(|s| s.get_name().into()).collect()
    }

    fn suitable_for_path_complete(line: &str) -> bool {
        let line = line.trim();
        line.starts_with("load") || line.starts_with("write") || line.starts_with("format")
    }
}
// helper:1 ends here

// [[file:../gosh.note::*pub][pub:1]]
#[derive(Clap, Debug)]
struct Gosh {
    /// Execute gosh script
    #[clap(short = 'x')]
    script_file: Option<PathBuf>,

    #[clap(flatten)]
    verbose: gut::cli_clap::Verbosity,
}

pub fn repl_enter_main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // entry shell mode or subcommands mode
    if args.len() > 1 {
        let args = Gosh::parse();
        args.verbose.setup_logger();

        if let Some(script_file) = &args.script_file {
            info!("Execute script file: {:?}", script_file);
            Interpreter::new().interpret_script_file(script_file)?;
        } else {
            info!("Reading batch script from stdin ..");
            use std::io::{self, Read};

            let mut buffer = String::new();
            std::io::stdin().read_to_string(&mut buffer)?;
            Interpreter::new().interpret_script(&buffer)?;
        }
    } else {
        Interpreter::new().start_repl()?;
    }

    Ok(())
}
// pub:1 ends here
