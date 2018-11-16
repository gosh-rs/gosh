// cmd loop

// [[file:~/Workspace/Programming/gosh/gosh.note::*cmd%20loop][cmd loop:1]]
use quicli::main;
use quicli::prelude::*;

use linefeed::{Interface, ReadResult};

mod cli;
use crate::cli::Commander;

use std::path::{PathBuf};
use dirs;

fn get_history_file() -> Result<PathBuf> {
    match dirs::home_dir() {
        Some(path) => {
            let filename = path.join(".gosh.history");
            Ok(filename)
        },
        None => bail!("Impossible to get your home dir!"),
    }
}

main!({
    let mut reader = Interface::new("rusty gosh")?;

    let version = env!("CARGO_PKG_VERSION");
    println!("This is the rusty gosh shell version {}.", version);
    println!("Enter \"help\" or \"?\" for a list of commands.");
    println!("Press Ctrl-D or enter \"quit\" or \"q\" to exit.");
    println!("");

    reader.set_prompt("gosh> ");

    let mut commander = Commander::new();
    let history_file = get_history_file().unwrap();
    if let Err(e) = reader.load_history(&history_file) {
        if e.kind() == std::io::ErrorKind::NotFound {
            println!("History file {} doesn't exist, not loading history.", history_file.display());
        } else {
            eprintln!("Could not load history file {}: {}", history_file.display(), e);
        }
    }

    while let ReadResult::Input(line) = reader.read_line()? {
        if !line.trim().is_empty() {
            reader.add_history(line.clone());
        }

        let (cmd, args) = split_first_word(&line);

        match cmd {
            "help" | "?" => {
                println!("gosh subcommands:");
                println!();
                for &(cmd, help) in GOSH_COMMANDS {
                    println!("  {:16} - {}", cmd, help);
                }
                println!();
            },
            "load" => {
                if args.is_empty() {
                    println!("Please input path to a file containing molecule.");
                } else {
                    let filename = args;
                    if let Err(ref e) = &mut commander.load(filename) {
                        eprintln!("{:?}", e);
                    } else {
                        println!("{} molecules loaded from: {:?}.", commander.molecules.len(), filename);
                    }
                }
            },

            "rebond" => {
                if let Err(ref e) = &mut commander.rebond() {
                    eprintln!("{:?}", e);
                }
            },

            "clean" => {
                if let Err(ref e) = &mut commander.clean() {
                    eprintln!("{:?}", e);
                }
            },

            "write" => {
                if args.is_empty() {
                    println!("Please input path to save the molecule.");
                } else {
                    let filename = args;
                    if let Err(ref e) = &commander.write(filename) {
                        eprintln!("{:?}", e);
                    }
                }
            },

            "format" => {
                if args.is_empty() {
                    println!("Please input path to user defined template file.");
                } else {
                    let filename = args;
                    if let Err(ref e) = &commander.format(filename) {
                        eprintln!("{:?}", e);
                    }
                }
            },

            "fragment" => {
                if let Err(ref e) = &mut commander.fragment() {
                    eprintln!("{:?}", e);
                } else {
                    println!("got {:} fragments", commander.molecules.len());
                }
            }

            "avail" => {
                if let Err(ref e) = &mut commander.avail() {
                    eprintln!("{:?}", e);
                }
            },

            "save" => {
                if let Err(ref e) = &commander.save() {
                    eprintln!("{:?}", e);
                } else {
                    println!("saved.");
                }
            },

            "ls" => {
                if let Err(ref e) = &commander.extern_cmdline("ls") {
                    eprintln!("{:?}", e);
                }
            },

            "pwd" => {
                if let Err(ref e) = &commander.extern_cmdline("pwd") {
                    eprintln!("{:?}", e);
                }
            },

            "quit" | "q" => {
                if let Err(e) = reader.save_history(&history_file) {
                    eprintln!("Could not save history file {}: {}", history_file.display(), e);
                }
                break;
            },

            "" => (),
            _ => println!("{:?}: not a command", line),
        }
    }
});

static GOSH_COMMANDS: &'static [(&'static str, &'static str)] = &[
    ("help",             "You're looking at it"),
    ("quit",             "Quit gosh"),
    ("load",             "Load molecule from disk"),
    ("write",            "Write molecules into file"),
    ("rebond",           "Rebuild bonds based on atom distances."),
    ("format",           "Format molecule using user defined template file."),
    ("clean",            "Clean up bad molecular geometry."),
    ("avail",            "Show supported file formats."),
    ("fragment",         "Break molecule into smaller fragments based on connectivity."),
];

fn split_first_word(s: &str) -> (&str, &str) {
    let s = s.trim();

    match s.find(|ch: char| ch.is_whitespace()) {
        Some(pos) => (&s[..pos], s[pos..].trim_left()),
        None => (s, "")
    }
}
// cmd loop:1 ends here
