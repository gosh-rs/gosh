// [[file:~/Workspace/Programming/gosh/gosh.note::c5024615-a25b-4b40-9305-890be0fe004b][c5024615-a25b-4b40-9305-890be0fe004b]]
extern crate linefeed;
use linefeed::{Reader, ReadResult};

fn main() {
    let mut reader = Reader::new("rusty gosh").unwrap();

    println!("This is the gosh shell.");
    println!("Enter \"help\" or \"?\" for a list of commands.");
    println!("Press Ctrl-D or enter \"quit\" or \"q\" to exit.");
    println!("");

    reader.set_prompt("gosh> ");

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
            "quit" | "q" => break,
            "" => (),
            _ => println!("{:?}: not a command", line),
        }
    }
}

static GOSH_COMMANDS: &'static [(&'static str, &'static str)] = &[
    ("help",             "You're looking at it"),
    ("quit",             "Quit the demo"),
    ("load",             "Load molecule from disk"),
];

fn split_first_word(s: &str) -> (&str, &str) {
    let s = s.trim();

    match s.find(|ch: char| ch.is_whitespace()) {
        Some(pos) => (&s[..pos], s[pos..].trim_left()),
        None => (s, "")
    }
}
// c5024615-a25b-4b40-9305-890be0fe004b ends here
