//! # Fake Words Generator
//!
//! A Markov chain-based fake word generator that produces pronounceable pseudo-words
//! based on character transition probabilities.
//!
//! This library uses a second-order Markov chain model trained on real words to generate
//! realistic-looking fake words.
//!
//! The probability weights are pre-computed and embedded in the binary, making the
//! generator self-contained with no external dependencies at runtime.
//!
//! ## Features
//!
//! - **Fast generation**: Uses Walker's alias method for O(1) weighted random selection
//! - **Embedded weights**: No external files needed at runtime
//! - **Pronounceable output**: Generates words that look and sound like real words
//! - **Thread-safe**: `Chooser` implements `Send` and `Sync`
//!
//! ## Quick Start
//!
//! ```rust
//! use flabild::Chooser;
//!
//! // Create a generator
//! let chooser = Chooser::new();
//!
//! // Generate a fake word
//! let word = chooser.word().expect("Failed to generate word");
//! println!("Generated word: {}", word);
//! ```
//!
//! ## Examples
//!
//! ### Basic Usage
//!
//! ```rust
//! use flabild::Chooser;
//!
//! let chooser = Chooser::new();
//! let word = chooser.word().unwrap();
//! assert!(!word.is_empty());
//! println!("Random fake word: {}", word);
//! ```
//!
//! ### Generating Multiple Words
//!
//! ```rust
//! use flabild::Chooser;
//!
//! let chooser = Chooser::new();
//!
//! // Generate 10 fake words
//! for i in 1..=10 {
//!     match chooser.word() {
//!         Ok(word) => println!("{}. {}", i, word),
//!         Err(e) => eprintln!("Error generating word {}: {}", i, e),
//!     }
//! }
//! ```
//!
//! ### Error Handling
//!
//! ```rust
//! use flabild::Chooser;
//!
//! let chooser = Chooser::new();
//!
//! match chooser.word() {
//!     Ok(word) => println!("Generated: {}", word),
//!     Err(e) => {
//!         // Handle errors (missing weights or exceeded max length)
//!         eprintln!("Word generation failed: {}", e);
//!     }
//! }
//! ```
//!
//! ### Using `try_new` for Fallible Initialization
//!
//! ```rust
//! use flabild::Chooser;
//!
//! // Use try_new if you want to handle initialization errors
//! match Chooser::try_new() {
//!     Ok(chooser) => {
//!         let word = chooser.word().unwrap();
//!         println!("Generated: {}", word);
//!     }
//!     Err(e) => {
//!         eprintln!("Failed to initialize chooser: {}", e);
//!     }
//! }
//! ```
//!
//! ### Reusing the Generator
//!
//! ```rust
//! use flabild::Chooser;
//!
//! // Create once and reuse - more efficient than creating multiple times
//! let chooser = Chooser::new();
//!
//! let words: Vec<String> = (0..100)
//!     .filter_map(|_| chooser.word().ok())
//!     .collect();
//!
//! println!("Generated {} words", words.len());
//! ```

use std::collections::HashMap;

use anyhow::{Context, Result, anyhow};
use weighted_rand::builder::*;

/// Available characters in the alphabet, including word boundary markers.
///
/// The alphabet consists of:
/// - `_` (underscore): Word boundary marker used at start
/// - `a-z`: Standard lowercase English letters
/// - `|` (pipe): Terminator character marking end of word
pub const CHARS: [char; 28] = [
    '_', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
    's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '|',
];

const WEIGHTS: &[u8] = include_bytes!("data/weights.cbor");
const ALPHABET_SIZE: usize = CHARS.len();
const INITIAL_PAIR: Pair = ['_', '_'];
const TERMINATOR: char = '|';

/// A character pair used as a Markov chain state.
///
/// The generator uses pairs of characters to determine what character should come next,
/// making it a second-order Markov chain. For example, after seeing "th", different
/// characters have different probabilities (e is very likely, x is very unlikely).
pub type Pair = [char; 2];

/// Probability weights for each character in the alphabet.
///
/// Each position corresponds to a character in [`CHARS`]. Higher weights mean
/// higher probability of that character being selected.
pub type Weights = [u32; ALPHABET_SIZE];

/// Maps character pairs to their successor probability distributions.
///
/// This is the core data structure that drives word generation. Each pair of characters
/// maps to an array of weights indicating how likely each character in the alphabet is
/// to follow that pair.
pub type Choices = HashMap<Pair, Weights>;

/// A fake words generator using Markov chains.
///
/// `Chooser` generates pronounceable pseudo-words by modeling character transition
/// probabilities. It's initialized with pre-trained weights and can generate unlimited
/// words efficiently.
///
/// # Thread Safety
///
/// `Chooser` is both `Send` and `Sync`, making it safe to share across threads.
/// The internal `HashMap` is read-only after initialization.
///
/// # Examples
///
/// ```rust
/// use flabild::Chooser;
///
/// let chooser = Chooser::new();
/// let word = chooser.word().expect("Failed to generate word");
/// println!("Generated: {}", word);
/// ```
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
    /// Creates a `Chooser` from embedded weights.
    ///
    /// This is the standard way to create a `Chooser`. It loads pre-computed probability
    /// weights from data embedded in the binary at compile time.
    ///
    /// # Panics
    ///
    /// Panics if the embedded weights data is corrupted or cannot be deserialized.
    /// This should only happen if the binary has been corrupted or if there's a version
    /// mismatch between the weights data format and the deserialization code.
    ///
    /// If you need to handle initialization errors gracefully, use [`try_new`](Self::try_new) instead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flabild::Chooser;
    ///
    /// let chooser = Chooser::new();
    /// // Ready to generate words
    /// ```
    pub fn new() -> Chooser {
        Self::try_new().expect("Failed to initialize Chooser")
    }

    /// Attempts to create a `Chooser` from embedded weights.
    ///
    /// This is the fallible version of [`new`](Self::new). Use this if you want to handle
    /// initialization errors instead of panicking.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The embedded weights data is corrupted
    /// - The data format doesn't match what `serde_cbor` expects
    /// - There's a deserialization failure for any other reason
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flabild::Chooser;
    ///
    /// match Chooser::try_new() {
    ///     Ok(chooser) => {
    ///         println!("Chooser initialized successfully");
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Failed to initialize: {}", e);
    ///     }
    /// }
    /// ```
    pub fn try_new() -> Result<Self> {
        let choices: Choices = serde_cbor::from_slice(WEIGHTS)
            .context("Failed to deserialize character pair weights")?;
        Ok(Chooser { choices })
    }

    /// Generates a random fake word.
    ///
    /// Uses the Markov chain model to generate a pronounceable pseudo-word. The word
    /// starts from an initial state and continues adding characters based on probability
    /// weights until a terminator is reached.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Weights are missing for a character pair encountered during generation
    ///   (indicates corrupted or incomplete training data)
    /// - Generation exceeds the maximum word length of 80 characters
    ///   (should be extremely rare with properly trained weights)
    ///
    /// # Examples
    ///
    /// ## Basic generation
    ///
    /// ```rust
    /// use flabild::Chooser;
    ///
    /// let chooser = Chooser::new();
    /// let word = chooser.word()?;
    /// println!("Generated: {}", word);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// ## Handling errors
    ///
    /// ```rust
    /// use flabild::Chooser;
    ///
    /// let chooser = Chooser::new();
    ///
    /// match chooser.word() {
    ///     Ok(word) => println!("Success: {}", word),
    ///     Err(e) => eprintln!("Generation failed: {}", e),
    /// }
    /// ```
    ///
    /// ## Batch generation
    ///
    /// ```rust
    /// use flabild::Chooser;
    ///
    /// let chooser = Chooser::new();
    /// let words: Result<Vec<String>, _> = (0..10)
    ///     .map(|_| chooser.word())
    ///     .collect();
    ///
    /// match words {
    ///     Ok(words) => println!("Generated {} words", words.len()),
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn word(&self) -> Result<String> {
        const MAX_WORD_LENGTH: usize = 80; // Should be long enough

        let mut word = String::with_capacity(16); // Again, should be long enough
        let mut pair = INITIAL_PAIR;

        for _ in 0..MAX_WORD_LENGTH {
            let pair_weights = self
                .choices
                .get(&pair)
                .with_context(|| format!("Missing weights for pair: {:?}", pair))?;

            let wa_table = WalkerTableBuilder::new(pair_weights).build();
            let next_char = CHARS[wa_table.next()];
            if next_char == TERMINATOR {
                return Ok(word);
            }

            word.push(next_char);
            pair = [pair[1], next_char];
        }

        Err(anyhow!(
            "Word generation exceeded maximum length of {}",
            MAX_WORD_LENGTH
        ))
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
