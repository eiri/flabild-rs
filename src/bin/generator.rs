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

    map_words(&words)
}

fn map_words(words: &str) -> Result<Choices> {
    words
        .par_lines()
        .try_fold(Choices::new, |mut acc, word| {
            // Reject dictionary markers and non-ASCII letters before counting.
            if let Some(letter) = word.chars().find(|c| !c.is_ascii_lowercase()) {
                return Err(anyhow!("unsupported character {letter:?} in word {word:?}"));
            }

            let mut triplet = ['_', '_', '_'];
            for idx in word
                .bytes()
                .map(|b| (b - b'a' + 1) as usize)
                .chain(std::iter::once(27))
            {
                let letter = CHARS[idx];
                triplet[0] = triplet[1];
                triplet[1] = triplet[2];
                triplet[2] = letter;

                let pair = [triplet[0], triplet[1]];
                acc.entry(pair).or_insert([0u32; 28])[idx] += 1;
            }
            Ok(acc)
        })
        .try_reduce(Choices::new, |mut acc, other| {
            for (pair, weights) in other {
                let weights_acc = acc.entry(pair).or_insert([0u32; 28]);
                for (i, &count) in weights.iter().enumerate() {
                    weights_acc[i] += count;
                }
            }
            Ok(acc)
        })
}

fn store_map(config: &Config, choices: &Choices) -> Result<()> {
    let path = &config.out_path;

    let bytes = serde_cbor::to_vec(choices)?;
    fs::write(path, bytes)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_chars() {
        for word in ["World", "café", "can't", "two words", "a_b", "a|b"] {
            let input = format!("hello\n{word}\n");
            let err = map_words(&input).unwrap_err().to_string();
            let letter = word.chars().find(|c| !c.is_ascii_lowercase()).unwrap();

            assert_eq!(
                err,
                format!("unsupported character {letter:?} in word {word:?}")
            );
        }
    }

    #[test]
    fn counts_valid_words() {
        let choices = map_words("ab\r\nab\r\n").unwrap();

        assert_eq!(choices.len(), 3);
        assert_eq!(choices[&['_', '_']][1], 2);
        assert_eq!(choices[&['_', 'a']][2], 2);
        assert_eq!(choices[&['a', 'b']][27], 2);
        assert!(
            choices
                .values()
                .all(|weights| weights.iter().sum::<u32>() == 2)
        );
    }
}
