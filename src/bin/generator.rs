use std::collections::BTreeMap;
use std::{
    env,
    error::Error,
    fs::{self, File},
    io::Write,
    process,
};

use flabild::{CHARS, Pair, Weights};

type Triplet = [char; 3];

struct Config {
    dict_path: String,
    out_path: String,
}

impl Config {
    fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        // drop script name
        args.next();

        let dict_path = match args.next() {
            Some(arg) => arg,
            None => return Err("missing path to the dictionary"),
        };

        let out_path = match args.next() {
            Some(arg) => arg,
            None => return Err("missing path to the output file"),
        };

        Ok(Config {
            dict_path,
            out_path,
        })
    }
}

fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|e| {
        eprintln!("can't parse arguments: {e}");
        process::exit(1);
    });

    let reduce_map = generate_map(&config).unwrap_or_else(|e| {
        eprintln!("can't generate pairs: {e}");
        process::exit(1);
    });

    let mut out = File::create(config.out_path).unwrap_or_else(|e| {
        eprintln!("can't create output file: {e}");
        process::exit(1);
    });

    writeln!(out, "use crate::PairMap;\n").unwrap();
    writeln!(
        out,
        "pub fn build_choices_map() -> PairMap {{\nPairMap::from(["
    )
    .unwrap();
    for (pair, weights) in reduce_map {
        writeln!(out, "({:?}, {:?}),", pair, weights).unwrap();
    }
    writeln!(out, "])\n}}").unwrap();
}

fn generate_map(config: &Config) -> Result<BTreeMap<Pair, Weights>, Box<dyn Error>> {
    let words = fs::read_to_string(&config.dict_path)?;

    // Map
    let mut map_sink: Vec<Triplet> = Vec::new();
    for word in words.lines() {
        let mut triplet = ['_', '_', '_'];
        for letter in word.chars().chain(std::iter::once('|')) {
            triplet[0] = triplet[1];
            triplet[1] = triplet[2];
            triplet[2] = letter;
            map_sink.push(triplet);
        }
    }

    // Reduce
    let mut reduce_map = BTreeMap::new();
    for triplet in map_sink {
        let pair = [triplet[0], triplet[1]];
        let idx = CHARS.iter().position(|&c| c == triplet[2]).unwrap();

        let weights = reduce_map.entry(pair).or_insert([0u32; 28]);
        weights[idx] += 1;
    }

    Ok(reduce_map)
}
