//! Bloodless card descriptions can use rich text instead of just Strings. This text may contain links to other cards, or represent a Saga.
use super::CardId;
use std::{
    fmt::Display,
    slice::{Iter, IterMut},
    vec::IntoIter,
};

use serde::{
    de::Visitor,
    ser::{SerializeSeq, SerializeStruct},
    Deserialize, Serialize,
};

/// An element of `RichString`s
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RichElement {
    String(String),
    CardId { display: String, identity: CardId },
    SpecificCard { display: String, id: String },
    CardSearch { display: String, search: String },
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

impl<'de> Deserialize<'de> for RichString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DeVisitor;

        impl<'de> Visitor<'de> for DeVisitor {
            type Value = RichString;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("A string, an array of rich string stuff")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(RichString {
                    elements: vec![RichElement::String(v.to_string())],
                })
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut vec = vec![];
                while let Some(el) = seq.next_element()? {
                    vec.push(el);
                }
                Ok(RichString { elements: vec })
            }
        }
        deserializer.deserialize_any(DeVisitor)
    }
}

impl Serialize for RichString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.elements.first() {
            Some(RichElement::String(x)) if self.elements.len() == 1 => serializer.serialize_str(x),
            None => serializer.serialize_str(""),
            _ => {
                let mut seq = serializer.serialize_seq(Some(self.elements.len()))?;
                for x in &self.elements {
                    seq.serialize_element(&x)?;
                }
                seq.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for RichElement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DeVisitor;

        impl<'de> Visitor<'de> for DeVisitor {
            type Value = RichElement;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("A string, an array of strings, or other stuff")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v == "\n" || v == "\r\n" {
                    Ok(RichElement::LineBreak)
                } else {
                    Ok(RichElement::String(v.to_string()))
                }
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut vec = vec![];
                while let Some(element) = seq.next_element()? {
                    vec.push(element);
                }
                Ok(RichElement::Saga(vec))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut id = None;
                let mut identity = None;
                let mut display = None;
                let mut search = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "display" => display = map.next_value()?,
                        "identity" => identity = map.next_value()?,
                        "search" => search = map.next_value()?,
                        "id" => id = map.next_value()?,
                        field => {
                            return Err(serde::de::Error::unknown_field(
                                field,
                                &["display", "identity", "id"],
                            ))
                        }
                    }
                }

                match (display, identity, id, search) {
                    (None, _, _, _) => Err(serde::de::Error::missing_field("display")),
                    (Some(_), Some(_), Some(_), Some(_)) => Err(serde::de::Error::custom(
                        "expected something with either id or identity",
                    )),
                    (Some(display), None, None, Some(search)) => {
                        Ok(RichElement::CardSearch { display, search })
                    }
                    (Some(display), Some(identity), None, None) => {
                        Ok(RichElement::CardId { display, identity })
                    }
                    (Some(display), None, Some(id), None) => {
                        Ok(RichElement::SpecificCard { display, id })
                    }
                    (Some(_), None, None, None) => Err(serde::de::Error::missing_field(
                        "either id or identity or search",
                    )),
                    _ => Err(serde::de::Error::custom("what are you DOING")),
                }
            }
        }

        deserializer.deserialize_any(DeVisitor)
    }
}

impl Serialize for RichElement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::String(string) => serializer.serialize_str(string),
            Self::CardId { display, identity } => {
                let mut map = serializer.serialize_struct("CardId", 2)?;
                map.serialize_field("display", display)?;
                map.serialize_field("identity", identity)?;
                map.end()
            }
            Self::SpecificCard { display, id } => {
                let mut map = serializer.serialize_struct("Card", 2)?;
                map.serialize_field("display", display)?;
                map.serialize_field("id", id)?;
                map.end()
            }
            Self::CardSearch { display, search } => {
                let mut map = serializer.serialize_struct("Search", 2)?;
                map.serialize_field("display", display)?;
                map.serialize_field("search", search)?;
                map.end()
            }
            Self::Saga(strings) => {
                let mut seq = serializer.serialize_seq(Some(strings.len()))?;
                for a in strings {
                    seq.serialize_element(a)?;
                }
                seq.end()
            }
            Self::LineBreak => serializer.serialize_str("\n"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{cards::Card, search};

    #[test]
    fn test_serialize() {
        let card: Card = serde_json::from_str(
            &std::fs::read_to_string("tests/rich.json").expect("Couldn't load rich.json"),
        )
        .expect("Couldn't convert rich.json to a card");

        println!("{card}");
    }

    #[test]
    fn test_real_cards() {
        let card: Vec<Card> = serde_json::from_str(
            &std::fs::read_to_string("../hemolymph/server/cards.json")
                .expect("Couldn't load the real cards.json"),
        )
        .expect("Couldn't convert the real cards.json to a card");

        let card = search::search(
            &search::query_parser::query_parser("shuffle a grand design").unwrap(),
            card.iter(),
        );

        for card in card {
            println!("{card}");
        }
    }
}
