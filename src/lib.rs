use std::collections::HashMap;

use weighted_rand::builder::*;

mod choices;

pub const CHARS: [char; 28] = [
    '_', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
    's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '|',
];

pub type Pair = [char; 2];

pub type Weights = [u32; 28];

pub type Choices = HashMap<Pair, Weights>;

#[derive(Debug)]
pub enum FlabildError {
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
        let choices = choices::build_choices();
        Chooser { choices }
    }

    pub fn word(&self) -> Result<String, FlabildError> {
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
