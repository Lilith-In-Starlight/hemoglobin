//! Bloodless card descriptions can use rich text instead of just Strings. This text may contain links to other cards, or represent a Saga.
mod serde;
use super::CardId;
use std::{
    fmt::Display,
    slice::{Iter, IterMut},
    vec::IntoIter,
};

/// An element of `RichString`s
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RichElement {
    String(String),
    CardId {
        display: String,
        identity: Box<CardId>,
    },
    SpecificCard {
        display: String,
        id: String,
    },
    CardSearch {
        display: String,
        search: String,
    },
    Saga(Vec<RichString>),
    LineBreak,
}

/// A rich text string. This exists so I can make a serde implementation
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct RichString {
    pub elements: Vec<RichElement>,
}

impl IntoIterator for RichString {
    type Item = RichElement;

    type IntoIter = IntoIter<RichElement>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl<'a> IntoIterator for &'a RichString {
    type Item = &'a RichElement;

    type IntoIter = Iter<'a, RichElement>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.iter()
    }
}

impl<'a> IntoIterator for &'a mut RichString {
    type Item = &'a mut RichElement;

    type IntoIter = IterMut<'a, RichElement>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.iter_mut()
    }
}

impl RichString {
    pub fn iter(&self) -> std::slice::Iter<'_, RichElement> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, RichElement> {
        self.into_iter()
    }

    pub fn push_string(&mut self, str: String) {
        self.elements.push(RichElement::String(str));
    }
}

impl Display for RichElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(x) => write!(f, "{x}"),
            Self::CardId {
                display,
                identity: _,
            }
            | Self::SpecificCard { display, id: _ }
            | Self::CardSearch { display, search: _ } => write!(f, "{display}"),
            Self::Saga(elements) => {
                for element in elements {
                    writeln!(f, "{element}")?;
                }
                Ok(())
            }
            Self::LineBreak => writeln!(f),
        }
    }
}

impl Display for RichString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for element in &self.elements {
            write!(f, "{element}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::cards::Card;

    #[test]
    fn test_serialize() {
        let card: Card = serde_json::from_str(
            &std::fs::read_to_string("tests/rich.json").expect("Couldn't load rich.json"),
        )
        .expect("Couldn't convert rich.json to a card");

        println!("{card}");
    }
}
