#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

//! Hemolymph is a library containing data structures and functions useful for the card game Bloodless. It is used by [Hemolymph](http://hemolymph.net), the official card search engine.
//!
//! The two datastructures are `Card`, which represents a card and `CardID`, which represents a card identity. Card identities in this library do not represent card identities as defined by the game's rules, but rather as a more general structure for identifying cards.
//!
//! This library contains the search functions used by Hemolymph.

pub mod cards;
pub mod numbers;
pub mod search;

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
            fs::read_to_string("../hemolymph/server/cards.json").expect("Unable to read file");
        let cards: Vec<Card> = serde_json::from_str(&data).expect("Unable to parse JSON");
        let parsed = query_parser::query_parser("c!=2 n:default").unwrap();
        println!("{parsed}");
        // println!("{parsed:#?}");
        let cards = PrintableCards(search(&parsed, cards.iter()));
        println!("{cards}");
    }

    #[test]
    fn test_serialize() {
        let string =
            fs::read_to_string("../hemolymph/server/cards.json").expect("Unable to read file");
        let cards1: Vec<Card> = serde_json::from_str(&string).expect("Unable to parse JSON");
        let data = serde_json::to_string_pretty(&cards1).unwrap();
        let cards2: Vec<Card> = serde_json::from_str(&data).expect("Unable to parse JSON");
        assert_eq!(cards1, cards2);
    }
}
