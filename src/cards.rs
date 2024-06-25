use rand::prelude::SliceRandom;
use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::search::{Comparison, QueryRestriction};

/// Data structure for Cards. All fields are mandatory.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct Card {
    /// A value that uniquely identifies the card. This is necessary because many cards may have the same name.
    pub id: String,
    /// The card's name.
    pub name: String,
    /// Image names that the card may use. If this is empty, the name is used to generate an image name.
    #[serde(default)]
    pub img: Vec<String>,
    /// The card's text, excluding cost, stats and typeline.
    pub description: String,
    /// The card's blood cost.
    pub cost: usize,
    /// The card's health.
    pub health: usize,
    /// The card's overkill protection.
    pub defense: usize,
    /// The card's power.
    pub power: usize,
    /// The card's type (as per the game).
    pub r#type: String,
    #[serde(default)]
    /// Keywords the card's text has.
    pub keywords: Vec<Keyword>,
    #[serde(default)]
    /// Kins of the card, must include parent kins.
    pub kins: Vec<String>,
    #[serde(default)]
    /// Will be used to provide an official interpretation of the card's text.
    pub abilities: Vec<String>,
    #[serde(default)]
    /// Artists who made the card's art.
    pub artists: Vec<String>,
    /// What set the card belongs to.
    pub set: String,
    /// Where is the card legal.
    pub legality: HashMap<String, String>,
    #[serde(default)]
    /// Other tags you might add to the card.
    pub other: Vec<String>,
    #[serde(default)]
    /// What the card can be used for.
    pub functions: Vec<String>,
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut name = self.name.as_str();
        if name.len() > 20 {
            name = &name[0..18];
        }
        let mut nameline = name.to_owned();
        for _ in nameline.len()..24 {
            nameline.push(' ');
        }
        nameline.push_str(&self.cost.to_string());
        writeln!(f, "{nameline}")?;
        writeln!(f)?;
        writeln!(f, "{}", self.description)
    }
}

/// This trait is used in card generics. It is useful when you want a function to accept `CardID`s and not only `Card`s.
pub trait ReadProperties {
    /// Return a card's numeric property, if it has it.
    fn get_num_property(&self, property: &NumberProperties) -> Option<usize>;
    /// Return a card's text property, if it has it.
    fn get_str_property(&self, property: &StringProperties) -> Option<&str>;
    /// Return a card's array property, if it has it.
    fn get_vec_property(&self, property: &ArrayProperties) -> Option<&[String]>;
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
}

impl ReadProperties for Card {
    /// Return a card's numeric property, if it has it.
    /// Will only return None if the card's type contains the word "command" and the given value is not `NumberProperties::Cost`.
    fn get_num_property(&self, property: &NumberProperties) -> Option<usize> {
        match property {
            NumberProperties::Cost => Some(self.cost),
            NumberProperties::Health => {
                if self.r#type.contains("command") {
                    None
                } else {
                    Some(self.health)
                }
            }
            NumberProperties::Defense => {
                if self.r#type.contains("command") {
                    None
                } else {
                    Some(self.defense)
                }
            }
            NumberProperties::Power => {
                if self.r#type.contains("command") {
                    None
                } else {
                    Some(self.power)
                }
            }
        }
    }

    /// Return a card's text property, if it has it.
    /// Always returns Some.
    fn get_str_property(&self, property: &StringProperties) -> Option<&str> {
        Some(match property {
            StringProperties::Id => &self.id,
            StringProperties::Name => &self.name,
            StringProperties::Type => &self.r#type,
            StringProperties::Description => &self.description,
        })
    }

    /// Return a card's array property, if it has it.
    /// Always returns Some.
    fn get_vec_property(&self, property: &ArrayProperties) -> Option<&[String]> {
        Some(match property {
            ArrayProperties::Functions => &self.functions,
            ArrayProperties::Kins => &self.kins,
            ArrayProperties::Artists => &self.artists,
        })
    }

    /// Return a card's keywords. Always returns Some. If the card has no keywords, it will return Some empty array.
    fn get_keywords(&self) -> Option<&[Keyword]> {
        Some(&self.keywords)
    }

    /// Return a card's name. Always returns Some. If the card has no name, it will return Some empty string.
    /// Importantly, a `Card` should always have a name.
    fn get_name(&self) -> Option<&str> {
        Some(&self.name)
    }

    /// Return a card's description. Always returns Some. If the card has no description, it will return Some empty string.
    fn get_description(&self) -> Option<&str> {
        Some(&self.description)
    }

    /// Return a card's type. Always returns Some. If the card has no type, it will return Some empty string.
    /// Importantly, a `Card` should always have a type.
    fn get_type(&self) -> Option<&str> {
        Some(&self.r#type)
    }

    /// Return a card's kins. Always returns Some. If the card has no kins, it will return Some empty array.
    fn get_kins(&self) -> Option<&[String]> {
        Some(&self.kins)
    }
}

impl ReadProperties for &Card {
    /// Return a card's numeric property, if it has it.
    /// Will only return None if the card's type contains the word "command" and the given value is not `NumberProperties::Cost`.
    fn get_num_property(&self, property: &NumberProperties) -> Option<usize> {
        match property {
            NumberProperties::Cost => Some(self.cost),
            NumberProperties::Health => {
                if self.r#type.contains("command") {
                    None
                } else {
                    Some(self.health)
                }
            }
            NumberProperties::Defense => {
                if self.r#type.contains("command") {
                    None
                } else {
                    Some(self.defense)
                }
            }
            NumberProperties::Power => {
                if self.r#type.contains("command") {
                    None
                } else {
                    Some(self.power)
                }
            }
        }
    }

    /// Return a card's text property, if it has it.
    /// Always returns Some.
    fn get_str_property(&self, property: &StringProperties) -> Option<&str> {
        Some(match property {
            StringProperties::Id => &self.id,
            StringProperties::Name => &self.name,
            StringProperties::Type => &self.r#type,
            StringProperties::Description => &self.description,
        })
    }

    /// Return a card's array property, if it has it.
    /// Always returns Some.
    fn get_vec_property(&self, property: &ArrayProperties) -> Option<&[String]> {
        Some(match property {
            ArrayProperties::Functions => &self.functions,
            ArrayProperties::Kins => &self.kins,
            ArrayProperties::Artists => &self.artists,
        })
    }

    /// Return a card's keywords. Always returns Some. If the card has no keywords, it will return Some empty array.
    fn get_keywords(&self) -> Option<&[Keyword]> {
        Some(&self.keywords)
    }

    /// Return a card's name. Always returns Some. If the card has no name, it will return Some empty string.
    /// Importantly, a `Card` should always have a name.
    fn get_name(&self) -> Option<&str> {
        Some(&self.name)
    }

    /// Return a card's description. Always returns Some. If the card has no description, it will return Some empty string.
    fn get_description(&self) -> Option<&str> {
        Some(&self.description)
    }

    /// Return a card's type. Always returns Some. If the card has no type, it will return Some empty string.
    /// Importantly, a `Card` should always have a type.
    fn get_type(&self) -> Option<&str> {
        Some(&self.r#type)
    }

    /// Return a card's kins. Always returns Some. If the card has no kins, it will return Some empty array.
    fn get_kins(&self) -> Option<&[String]> {
        Some(&self.kins)
    }
}

impl ReadProperties for CardId {
    fn get_num_property(&self, property: &NumberProperties) -> Option<usize> {
        match property {
            NumberProperties::Cost => self.cost,
            NumberProperties::Health => {
                if self.r#type.as_ref().is_some_and(|x| x.contains("command")) {
                    None
                } else {
                    self.health
                }
            }
            NumberProperties::Defense => {
                if self.r#type.as_ref().is_some_and(|x| x.contains("command")) {
                    None
                } else {
                    self.defense
                }
            }
            NumberProperties::Power => {
                if self.r#type.as_ref().is_some_and(|x| x.contains("command")) {
                    None
                } else {
                    self.power
                }
            }
        }
    }

    fn get_str_property(&self, property: &StringProperties) -> Option<&str> {
        match property {
            StringProperties::Id => None,
            StringProperties::Name => self.name.as_deref(),
            StringProperties::Type => self.r#type.as_deref(),
            StringProperties::Description => self.description.as_deref(),
        }
    }

    fn get_vec_property(&self, property: &ArrayProperties) -> Option<&[String]> {
        match property {
            ArrayProperties::Functions => self.functions.as_deref(),
            ArrayProperties::Kins => self.kins.as_deref(),
            ArrayProperties::Artists => None,
        }
    }

    fn get_keywords(&self) -> Option<&[Keyword]> {
        self.keywords.as_deref()
    }

    fn get_name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn get_description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn get_type(&self) -> Option<&str> {
        self.r#type.as_deref()
    }

    fn get_kins(&self) -> Option<&[String]> {
        self.kins.as_deref()
    }
}

impl ReadProperties for &CardId {
    fn get_num_property(&self, property: &NumberProperties) -> Option<usize> {
        match property {
            NumberProperties::Cost => self.cost,
            NumberProperties::Health => {
                if self.r#type.as_ref().is_some_and(|x| x.contains("command")) {
                    None
                } else {
                    self.health
                }
            }
            NumberProperties::Defense => {
                if self.r#type.as_ref().is_some_and(|x| x.contains("command")) {
                    None
                } else {
                    self.defense
                }
            }
            NumberProperties::Power => {
                if self.r#type.as_ref().is_some_and(|x| x.contains("command")) {
                    None
                } else {
                    self.power
                }
            }
        }
    }

    fn get_str_property(&self, property: &StringProperties) -> Option<&str> {
        match property {
            StringProperties::Id => None,
            StringProperties::Name => self.name.as_deref(),
            StringProperties::Type => self.r#type.as_deref(),
            StringProperties::Description => self.description.as_deref(),
        }
    }

    fn get_vec_property(&self, property: &ArrayProperties) -> Option<&[String]> {
        match property {
            ArrayProperties::Functions => self.functions.as_deref(),
            ArrayProperties::Kins => self.kins.as_deref(),
            ArrayProperties::Artists => None,
        }
    }

    fn get_keywords(&self) -> Option<&[Keyword]> {
        self.keywords.as_deref()
    }

    fn get_name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn get_description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn get_type(&self) -> Option<&str> {
        self.r#type.as_deref()
    }

    fn get_kins(&self) -> Option<&[String]> {
        self.kins.as_deref()
    }
}

/// Data structure for card identities. These card identities are slightly more general than the concept within the game, as they allow you to match things that are only relevant for searching cards.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct CardId {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub keywords: Option<Vec<Keyword>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub kins: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defense: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub abilities: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub functions: Option<Vec<String>>,
}

/// A keyword may contain data. This data may be a string or a `CardID`.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum KeywordData {
    CardID(CardId),
    String(String),
}

/// A card's Keyword.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Keyword {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<KeywordData>,
}

/// A card's numerical properties
#[derive(Debug, Clone, Copy)]
pub enum NumberProperties {
    Cost,
    Health,
    Power,
    Defense,
}

impl Display for NumberProperties {
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
pub enum ArrayProperties {
    Functions,
    Kins,
    Artists,
}

impl Display for ArrayProperties {
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
pub enum StringProperties {
    Id,
    Name,
    Type,
    Description,
}

impl Display for StringProperties {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StringProperties::Id => write!(f, "ID"),
            StringProperties::Name => write!(f, "Name"),
            StringProperties::Type => write!(f, "Type"),
            StringProperties::Description => write!(f, "Description"),
        }
    }
}

impl Card {
    /// Obtains a randomly selected image name from the `Card`'s img field. If it can't, it gets an image name based on its name.
    #[must_use]
    pub fn get_image(&self) -> String {
        self.img
            .choose(&mut rand::thread_rng())
            .cloned()
            .unwrap_or(self.name.replace(' ', ""))
    }
}

impl CardId {
    #[must_use]
    /// Creates a vector of `QueryRestriction`s defined by the `CardID`.
    pub fn get_as_query(&self) -> Vec<QueryRestriction> {
        let mut restrictions = vec![];

        if let Some(name) = &self.name {
            restrictions.push(QueryRestriction::Contains(
                StringProperties::Name,
                name.clone(),
            ));
        }

        if let Some(kins) = &self.kins {
            for kin in kins {
                restrictions.push(QueryRestriction::Has(ArrayProperties::Kins, kin.clone()));
            }
        }

        if let Some(keywords) = &self.keywords {
            for keyword in keywords {
                restrictions.push(QueryRestriction::HasKw(keyword.name.clone()));
            }
        }

        if let Some(cost) = &self.cost {
            restrictions.push(QueryRestriction::Comparison(
                NumberProperties::Cost,
                Comparison::Equal(*cost),
            ));
        }

        if let Some(health) = &self.health {
            restrictions.push(QueryRestriction::Comparison(
                NumberProperties::Health,
                Comparison::Equal(*health),
            ));
        }

        if let Some(power) = &self.power {
            restrictions.push(QueryRestriction::Comparison(
                NumberProperties::Power,
                Comparison::Equal(*power),
            ));
        }

        if let Some(defense) = &self.defense {
            restrictions.push(QueryRestriction::Comparison(
                NumberProperties::Defense,
                Comparison::Equal(*defense),
            ));
        }

        restrictions
    }
}
