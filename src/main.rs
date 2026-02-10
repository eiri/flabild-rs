use std::collections::HashMap;

use clap::Parser;
use weighted_rand::builder::*;

mod pairs;

#[derive(Debug, Parser)]
#[command(version, about, long_about = "Generator of fake pronounceable words")]
struct Args {
    /// number of words to generate
    #[arg(short, long, default_value_t = 1)]
    number: u8,
}

const CHARS: [char; 28] = [
    '_', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
    's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '|',
];

#[derive(Debug)]
enum FlabildError {
    NotFound,
}

impl std::fmt::Display for FlabildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FlabildError::NotFound => write!(f, "pair not found"),
        }
    }
}

impl std::error::Error for FlabildError {}

type Pair = [char; 2];

type PairMap = HashMap<Pair, [u32; 28]>;

#[derive(Debug)]
struct Chooser {
    choices: PairMap,
}

impl Chooser {
    fn new() -> Chooser {
        let choices = pairs::build_choices_map();
        Chooser { choices }
    }

    fn word(&self) -> Result<String, FlabildError> {
        let mut word = String::new();
        let mut pair = ['_', '_'];
        loop {
            let pair_weights = self.choices.get(&pair).ok_or(FlabildError::NotFound)?;
            let builder = WalkerTableBuilder::new(pair_weights);
            let wa_table = builder.build();
            let r = CHARS[wa_table.next()];
            if r == '|' {
                break;
            }
            word.push(r);
            pair[0] = pair[1];
            pair[1] = r;
        }
        Ok(word)
    }
}

fn main() {
    let args = Args::parse();

    let chooser = Chooser::new();

    for _ in 0..args.number {
        match chooser.word() {
            Ok(word) => println!("{}", word),
            Err(e) => {
                eprintln!("error: {}", e);
                std::process::exit(1)
            }
        }
    }
}
