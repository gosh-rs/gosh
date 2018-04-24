// [[file:~/Workspace/Programming/gosh/gosh.note::c5024615-a25b-4b40-9305-890be0fe004b][c5024615-a25b-4b40-9305-890be0fe004b]]
// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate linefeed;
extern crate gchemol;
#[macro_use]
extern crate error_chain;

use std::rc::Rc;
use std::io::Write;
use linefeed::{Reader, ReadResult};
use linefeed::terminal::Terminal;
use linefeed::complete::{Completer, Completion};
use error_chain::ChainedError; // trait which holds `display_chain`

use gchemol::{
    Molecule,
};


mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain!{}
}

use errors::*;

fn main() {
    let mut reader = Reader::new("rusty gosh").unwrap();

    println!("This is the gosh shell.");
    println!("Enter \"help\" or \"?\" for a list of commands.");
    println!("Press Ctrl-D or enter \"quit\" or \"q\" to exit.");
    println!("");

    reader.set_completer(Rc::new(GOSHCompleter));
    reader.set_prompt("gosh> ");

    let mut commander = Commander::new();

    while let Ok(ReadResult::Input(line)) = reader.read_line() {
        if !line.trim().is_empty() {
            reader.add_history(line.clone());
        }

        let (cmd, args) = split_first_word(&line);

        match cmd {
            "help" | "?" => {
                println!("linefeed demo commands:");
                println!();
                for &(cmd, help) in GOSH_COMMANDS {
                    println!("  {:16} - {}", cmd, help);
                }
                println!();
            },
            "load" => {
                if args.len() == 0 {
                    println!("Please input path to a file containing molecule.");
                } else {
                    let filename = args;
                    if let Err(ref e) = &mut commander.load(filename) {
                        let stderr = &mut ::std::io::stderr();
                        let errmsg = "Error writing to stderr";

                        writeln!(stderr, "{}", e.display_chain()).expect(errmsg);
                    } else {
                        println!("Molecule loaded from: {:?}.", filename);
                    }
                }
            },

            "write" | "save" => {
                if args.len() == 0 {
                    println!("Please input path to save the molecule.");
                } else {
                    let filename = args;
                    if let Some(mol) = &mut commander.molecule {
                        mol.to_file(filename);
                        println!("Molecule wrote to: {:?}.", filename);
                    } else {
                        println!("No molecule available.");
                    }
                }
            },

            "quit" | "q" => break,
            "" => (),
            _ => println!("{:?}: not a command", line),
        }
    }
}

struct Commander {
    molecule: Option<Molecule>,
}

impl Commander {
    fn new() -> Self {
        Commander {
            molecule: None,
        }
    }

    fn load(&mut self, filename: &str) -> Result<()> {
        let mol = Molecule::from_file(filename).chain_err(|| "bad")?;
        self.molecule = Some(mol);

        Ok(())
    }
}


static GOSH_COMMANDS: &'static [(&'static str, &'static str)] = &[
    ("help",             "You're looking at it"),
    ("quit",             "Quit the demo"),
    ("load",             "Load molecule from disk"),
    ("write",            "Write molecules into file"),
];

fn split_first_word(s: &str) -> (&str, &str) {
    let s = s.trim();

    match s.find(|ch: char| ch.is_whitespace()) {
        Some(pos) => (&s[..pos], s[pos..].trim_left()),
        None => (s, "")
    }
}

struct GOSHCompleter;

impl<Term: Terminal> Completer<Term> for GOSHCompleter {
    fn complete(&self, word: &str, reader: &Reader<Term>,
            start: usize, _end: usize) -> Option<Vec<Completion>> {
        let line = reader.buffer();

        let mut words = line[..start].split_whitespace();

        match words.next() {
            // Complete command name
            None => {
                let mut compls = Vec::new();

                for &(cmd, _) in GOSH_COMMANDS {
                    if cmd.starts_with(word) {
                        compls.push(Completion::simple(cmd.to_owned()));
                    }
                }

                Some(compls)
            }
            // Complete command parameters
            Some("get") | Some("set") => {
                if words.count() == 0 {
                    let mut res = Vec::new();

                    for (name, _) in reader.variables() {
                        if name.starts_with(word) {
                            res.push(Completion::simple(name.to_owned()));
                        }
                    }

                    Some(res)
                } else {
                    None
                }
            }
            _ => None
        }
    }
}
// c5024615-a25b-4b40-9305-890be0fe004b ends here
