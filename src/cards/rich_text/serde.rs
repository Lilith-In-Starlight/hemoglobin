use serde::{
    Deserialize, Serialize,
    de::Visitor,
    ser::{SerializeSeq, SerializeStruct},
};

use crate::cards::rich_text::RichElement;

use super::RichString;

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
                            ));
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
