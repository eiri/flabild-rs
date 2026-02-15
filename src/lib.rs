use std::collections::HashMap;

use anyhow::{Result, anyhow};
use weighted_rand::builder::*;

pub const CHARS: [char; 28] = [
    '_', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
    's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '|',
];

const WEIGHTS: &[u8] = include_bytes!("data/weights.cbor");

pub type Pair = [char; 2];

pub type Weights = [u32; 28];

pub type Choices = HashMap<Pair, Weights>;

#[derive(Debug)]
pub struct Chooser {
    choices: Choices,
}

impl Default for Chooser {
    fn default() -> Self {
        Self::new()
    }
}

impl Chooser {
    pub fn new() -> Chooser {
        let choices = serde_cbor::from_slice(WEIGHTS).unwrap();
        Chooser { choices }
    }

    pub fn word(&self) -> Result<String> {
        let mut word = String::new();
        let mut pair = ['_', '_'];
        loop {
            let pair_weights = self
                .choices
                .get(&pair)
                .ok_or_else(|| anyhow!("pair not found"))?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chooser_new() {
        let chooser = Chooser::new();
        assert!(!chooser.choices.is_empty(), "Choices should be loaded");
    }

    #[test]
    fn test_chooser_default() {
        let chooser = Chooser::default();
        assert!(!chooser.choices.is_empty(), "Default should work");
    }

    #[test]
    fn test_new_equals_default() {
        let chooser1 = Chooser::new();
        let chooser2 = Chooser::default();
        assert_eq!(chooser1.choices.len(), chooser2.choices.len());
    }

    #[test]
    fn test_word_generates_successfully() {
        let chooser = Chooser::new();
        let result = chooser.word();
        assert!(result.is_ok(), "Should generate a word");
    }

    #[test]
    fn test_word_returns_non_empty_string() {
        let chooser = Chooser::new();
        let word = chooser.word().unwrap();
        assert!(!word.is_empty(), "Generated word should not be empty");
    }

    #[test]
    fn test_word_contains_only_valid_chars() {
        let chooser = Chooser::new();
        let word = chooser.word().unwrap();

        for c in word.chars() {
            assert!(
                CHARS[1..27].contains(&c), // Exclude start and terminate chars
                "Word should only contain valid characters, found: {}",
                c
            );
        }
    }

    #[test]
    fn test_multiple_word_generations() {
        let chooser = Chooser::new();

        for _ in 0..10 {
            let result = chooser.word();
            assert!(result.is_ok(), "Should be able to generate multiple words");
        }
    }

    #[test]
    fn test_word_variability() {
        let chooser = Chooser::new();
        let mut words = std::collections::HashSet::new();

        // Generate 100 words - at least some should be different
        for _ in 0..100 {
            if let Ok(word) = chooser.word() {
                words.insert(word);
            }
        }

        assert!(words.len() > 1, "Should generate varied words");
    }

    #[test]
    fn test_word_reasonable_length() {
        let chooser = Chooser::new();

        for _ in 0..20 {
            let word = chooser.word().unwrap();
            assert!(
                word.len() > 0 && word.len() < 100,
                "Word length should be reasonable, got: {}",
                word.len()
            );
        }
    }

    #[test]
    fn test_chars_constant_length() {
        assert_eq!(CHARS.len(), 28, "CHARS should have exactly 28 elements");
    }

    #[test]
    fn test_chars_starts_with_underscore() {
        assert_eq!(CHARS[0], '_', "First char should be underscore");
    }

    #[test]
    fn test_chars_ends_with_pipe() {
        assert_eq!(CHARS[27], '|', "Last char should be pipe");
    }

    #[test]
    fn test_initial_pair_exists_in_choices() {
        let chooser = Chooser::new();
        let initial_pair = ['_', '_'];
        assert!(
            chooser.choices.contains_key(&initial_pair),
            "Choices should contain the initial pair"
        );
    }

    #[test]
    fn test_chooser_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Chooser>();
    }

    #[test]
    fn test_chooser_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Chooser>();
    }

    #[test]
    fn test_debug_implementation() {
        let chooser = Chooser::new();
        let debug_str = format!("{:?}", chooser);
        assert!(
            debug_str.contains("Chooser"),
            "Debug output should contain struct name"
        );
    }
}
