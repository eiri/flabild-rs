use std::collections::BTreeMap;
use std::{
    env,
    fs::{self, File},
    io::Write,
    process,
};

use anyhow::{Result, anyhow};

use flabild::{CHARS, Pair, Weights};

type Triplet = [char; 3];

struct Config {
    dict_path: String,
    out_path: String,
}

impl Config {
    fn build(mut args: impl Iterator<Item = String>) -> Result<Config> {
        // drop script name
        args.next();

        let dict_path = args
            .next()
            .ok_or_else(|| anyhow!("missing path to the dictionary"))?;

        let out_path = args
            .next()
            .ok_or_else(|| anyhow!("missing path to the output file"))?;

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
        eprintln!("can't generate weights: {e}");
        process::exit(1);
    });

    store_map(&config, reduce_map).unwrap_or_else(|e| {
        eprintln!("can't store weights: {e}");
        process::exit(1);
    })
}

fn generate_map(config: &Config) -> Result<BTreeMap<Pair, Weights>> {
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

fn store_map(config: &Config, reduce_map: BTreeMap<Pair, Weights>) -> Result<()> {
    let mut out = File::create(&config.out_path)?;

    writeln!(out, "use crate::Choices;\n")?;
    writeln!(out, "pub fn build_choices() -> Choices {{\nChoices::from([")?;
    for (pair, weights) in reduce_map {
        writeln!(out, "({:?}, {:?}),", pair, weights)?;
    }
    writeln!(out, "])\n}}")?;

    Ok(())
}
