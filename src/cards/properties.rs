use std::fmt::Display;

use crate::numbers::MaybeImprecise;

use super::Keyword;

/// This trait is used in card generics. It is useful when you want a function to accept `CardID`s and not only `Card`s.
pub trait Read {
    /// Return a card's numeric property, if it has it.
    fn get_num_property(&self, property: &Number) -> Option<MaybeImprecise>;
    /// Return a card's text property, if it has it.
    fn get_text_property(&self, property: &Text) -> Option<&str>;
    /// Return a card's array property, if it has it.
    fn get_vec_property(&self, property: &Array) -> Option<&[String]>;
    /// Return a card's keywords, if it has them. It may not have them if it is a `CardID`.
    fn get_keywords(&self) -> Option<&[Keyword]>;
    /// Return a card's name, if it has one. It may not have one if it is a `CardID`.
    fn get_name(&self) -> Option<&str>;
    /// Return a card's text, if it has one. It may not have one if it is a `CardID`.
    fn get_description(&self) -> Option<&str>;
    /// Return a card's type, if it has one. It may not have one if it is a `CardID`.
    fn get_type(&self) -> Option<&str>;
    /// Return a card's kins, if it has them. It may not have them if it is a `CardID`.
    fn get_kins(&self) -> Option<&[String]>;
    /// Return a card's flavor text, if it has one. It may not have one if it is a `CardID`.
    fn get_flavor_text(&self) -> Option<&str>;
}

/// A card's numerical properties
#[derive(Debug, Clone, Copy)]
pub enum Number {
    Cost,
    Health,
    Power,
    Defense,
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cost => write!(f, "Cost"),
            Self::Health => write!(f, "Health"),
            Self::Power => write!(f, "Power"),
            Self::Defense => write!(f, "Defense"),
        }
    }
}

/// A card's array properties
#[derive(Debug, Clone, Copy)]
pub enum Array {
    Functions,
    Kins,
    Artists,
}

impl Display for Array {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Functions => write!(f, "Functions"),
            Self::Kins => write!(f, "Kins"),
            Self::Artists => write!(f, "Artists"),
        }
    }
}

/// A card's text properties
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum Text {
    Id,
    Name,
    Type,
    Description,
    FlavorText,
}

impl Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Text::Id => write!(f, "ID"),
            Text::Name => write!(f, "Name"),
            Text::Type => write!(f, "Type"),
            Text::Description => write!(f, "Description"),
            Text::FlavorText => write!(f, "FlavorText"),
        }
    }
}
