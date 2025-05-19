#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

//! Hemolymph is a library containing data structures and functions useful for the card game Bloodless. It is used by [Hemolymph](http://hemolymph.net), the official card search engine.
//!
//! The two datastructures are `Card`, which represents a card and `CardId`, which represents a card identity. Card identities in this library do not represent card identities as defined by the game's rules, but rather as a more general structure for identifying cards.
//!
//! This library contains the search functions used by Hemolymph.

pub mod cards;
pub mod numbers;

/// Only handles lowercase because it'll be applied after `to_lowercase`
#[must_use]
pub fn clean_ascii(string: &str) -> String {
    let string = string.to_lowercase();
    clean_ascii_keep_case(&string)
}

/// Only handles lowercase because it'll be applied after `to_lowercase`
#[must_use]
pub fn clean_ascii_keep_case(string: &str) -> String {
    let string = string.replace('ä', "a");
    let string = string.replace('ë', "e");
    let string = string.replace('ï', "i");
    let string = string.replace('ö', "o");
    let string = string.replace('"', "");
    let string = string.replace('\'', "");
    let string = string.replace('.', "");
    let string = string.replace(',', "");
    string.replace('ü', "u")
}
