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
    use crate::{
        cards::Card,
        search::{query_parser::query_parser, search},
    };

    #[test]
    fn test_equals_search() {
        let cards: Vec<Card> = serde_json::from_str(
            &std::fs::read_to_string("tests/search.json").expect("Couldn't load search.json"),
        )
        .expect("Couldn't convert search.json to a vec of cards");
        for val in 4..=6 {
            let result = search(
                &query_parser(&format!("c={val}")).expect("couldn't parse query"),
                cards.iter(),
            );

            let fail = match val {
                4 => result
                    .iter()
                    .any(|x| x.name == "eq 5" || x.name == "gteq 5" || x.name == "gt 5"),
                5 => result
                    .iter()
                    .any(|x| x.name == "lt 5" || x.name == "gt 5" || x.name == "neq 5"),
                6 => result
                    .iter()
                    .any(|x| x.name == "lt 5" || x.name == "lteq 5" || x.name == "eq 5"),
                _ => unreachable!(),
            };

            assert!(!fail);
        }
    }

    #[test]
    fn test_gt_search() {
        let cards: Vec<Card> = serde_json::from_str(
            &std::fs::read_to_string("tests/search.json").expect("Couldn't load search.json"),
        )
        .expect("Couldn't convert search.json to a vec of cards");
        for val in 4..=6 {
            let result = search(
                &query_parser(&format!("c>{val}")).expect("couldn't parse query"),
                cards.iter(),
            );

            let fail = match val {
                4 => result.iter().any(|x| x.name == "lt 5"),
                5 => result
                    .iter()
                    .any(|x| x.name == "lt 5" || x.name == "lteq 5" || x.name == "eq 5"),
                6 => result
                    .iter()
                    .any(|x| x.name == "lt 5" || x.name == "lteq 5" || x.name == "eq 5"),
                _ => unreachable!(),
            };

            assert!(!fail);
        }
    }

    #[test]
    fn test_gteq_search() {
        let cards: Vec<Card> = serde_json::from_str(
            &std::fs::read_to_string("tests/search.json").expect("Couldn't load search.json"),
        )
        .expect("Couldn't convert search.json to a vec of cards");
        for val in 4..=6 {
            let result = search(
                &query_parser(&format!("c>={val}")).expect("couldn't parse query"),
                cards.iter(),
            );

            let fail = match val {
                4 => false,
                5 => result.iter().any(|x| x.name == "lt 5"),
                6 => result
                    .iter()
                    .any(|x| x.name == "lt 5" || x.name == "lteq 5" || x.name == "eq 5"),
                _ => unreachable!(),
            };

            assert!(!fail);
        }
    }
}
