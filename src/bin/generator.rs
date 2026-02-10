use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::process;

const CHARS: [char; 28] = [
    '_', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
    's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '|',
];

type Triplet = [char; 3];

struct Config {
    dict_path: String,
    out_path: String,
}

impl Config {
    fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments");
        }
        let dict_path = args[1].clone();
        let out_path = args[2].clone();
        Ok(Config {
            dict_path,
            out_path,
        })
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::build(&args).unwrap_or_else(|e| {
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

    writeln!(out, "use std::collections::HashMap;\n").unwrap();
    writeln!(
        out,
        "pub fn build_choices_map() -> HashMap<[char; 2], [u32; 28]> {{\nHashMap::from(["
    )
    .unwrap();
    for (pair, weights) in reduce_map {
        writeln!(out, "({:?}, {:?}),", pair, weights).unwrap();
    }
    writeln!(out, "])\n}}").unwrap();
}

fn generate_map(config: &Config) -> Result<HashMap<[char; 2], [u32; 28]>, Box<dyn Error>> {
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
    let mut reduce_map = HashMap::new();
    for triplet in map_sink {
        let pair = [triplet[0], triplet[1]];
        let idx = CHARS.iter().position(|&c| c == triplet[2]).unwrap();

        let weights = reduce_map.entry(pair).or_insert([0u32; 28]);
        weights[idx] += 1;
    }

    Ok(reduce_map)
}
