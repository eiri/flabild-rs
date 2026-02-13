use std::{env, fs, process};

use anyhow::{Result, anyhow};
use rayon::prelude::*;

use flabild::{CHARS, Choices};

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

    store_map(&config, &reduce_map).unwrap_or_else(|e| {
        eprintln!("can't store weights: {e}");
        process::exit(1);
    })
}

fn generate_map(config: &Config) -> Result<Choices> {
    let words = fs::read_to_string(&config.dict_path)?;

    let reduce_map = words
        .par_lines()
        .fold(Choices::new, |mut acc, word| {
            let mut triplet = ['_', '_', '_'];
            for letter in word.chars().chain(std::iter::once('|')) {
                triplet[0] = triplet[1];
                triplet[1] = triplet[2];
                triplet[2] = letter;

                let pair = [triplet[0], triplet[1]];
                let idx = CHARS.iter().position(|&c| c == triplet[2]).unwrap();
                acc.entry(pair).or_insert([0u32; 28])[idx] += 1;
            }
            acc
        })
        .reduce(Choices::new, |mut acc, other| {
            for (pair, weights) in other {
                let weights_acc = acc.entry(pair).or_insert([0u32; 28]);
                for (i, &count) in weights.iter().enumerate() {
                    weights_acc[i] += count;
                }
            }
            acc
        });

    Ok(reduce_map)
}

fn store_map(config: &Config, choices: &Choices) -> Result<()> {
    let path = &config.out_path;

    let bytes = serde_cbor::to_vec(choices)?;
    fs::write(path, bytes)?;

    Ok(())
}
