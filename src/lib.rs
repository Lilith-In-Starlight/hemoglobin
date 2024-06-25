#![warn(clippy::pedantic)]

//! Hemolymph is a library containing data structures and functions useful for the card game Bloodless. It is used by [Hemolymph](http://hemolymph.net), the official card search engine.
//!
//! The two datastructures are `Card`, which represents a card and `CardID`, which represents a card identity. Card identities in this library do not represent card identities as defined by the game's rules, but rather as a more general structure for identifying cards.
//!
//! This library contains the search functions used by Hemolymph.

use std::{
    cmp::{max, min},
    ops::Not,
};

pub mod cards;
pub mod search;

/// Represents whether a query has been matched or not. This is not always a boolean value, but instead a ternary value, as cards may have undefined properties.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum QueryMatch {
    /// Card did not have the requested property.
    NotHave,
    /// Card had the requested property and it did not matched the requested value.
    NotMatch,
    /// Card had the requested property and it matched the requested value.
    Match,
}

impl QueryMatch {
    /// A ternary OR which outputs the highest-valued result between `self` and `b`, where a `Match` is considered highest and `NotHave` is considered lowest.
    #[must_use]
    pub fn or(self, b: Self) -> Self {
        max(self, b)
    }
    /// A ternary XOR which outputs the highest-valued result between `self` and `b`, if they are not equal.
    /// If both values are `Match` or `NotMatch`, the output will be `NotMatch`.
    /// If both values are `NotHave`, the output will be `NotHave`.
    /// If no value is Match and there is a `NotHave`, the output will be `NotHave`.
    #[must_use]
    pub fn xor(self, b: Self) -> Self {
        match (self, b) {
            (Self::Match, Self::Match) => Self::NotMatch,
            (Self::NotHave, Self::NotHave) => Self::NotHave,
            (Self::Match, Self::NotMatch | Self::NotHave)
            | (Self::NotMatch | Self::NotHave, Self::Match) => Self::Match,
            (Self::NotMatch | Self::NotHave, Self::NotMatch) | (Self::NotMatch, Self::NotHave) => {
                Self::NotMatch
            }
        }
    }
    /// A ternary AND which outputs the lowest-valued result between `self` and `b`, where a `Match` is considered highest and `NotHave` is considered lowest.
    #[must_use]
    pub fn and(self, b: Self) -> Self {
        min(self, b)
    }
}

impl Not for QueryMatch {
    type Output = QueryMatch;

    /// Ternary NOT where `NotHave` is considered opposite to itself.
    fn not(self) -> Self::Output {
        match self {
            Self::Match => Self::NotMatch,
            Self::NotMatch => Self::Match,
            Self::NotHave => Self::NotHave,
        }
    }
}

#[cfg(test)]
mod test {
    use std::{fmt::Display, fs};

    use crate::{
        cards::Card,
        search::{query_parser, search},
    };

    struct PrintableCards<'a>(Vec<&'a Card>);

    impl<'a> Display for PrintableCards<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            for card in &self.0 {
                writeln!(f, "{card}")?;
                writeln!(f)?;
            }
            Ok(())
        }
    }

    #[test]
    fn test_search() {
        let data =
            fs::read_to_string("../hemolymph-server/cards.json").expect("Unable to read file");
        let cards: Vec<Card> = serde_json::from_str(&data).expect("Unable to parse JSON");
        let parsed = query_parser::query_parser("n:infected OR dby:(n:\"infected fly\")").unwrap();
        println!("{parsed}");
        // println!("{parsed:#?}");
        let cards = PrintableCards(search(&parsed, cards.iter()));
        println!("{cards}");
    }
}
