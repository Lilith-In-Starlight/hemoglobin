pub mod properties;
pub mod rich_text;
use crate::cards::properties::Array;
use crate::cards::properties::Number;
use crate::cards::properties::Read;
use crate::cards::properties::Text;
use crate::numbers::MaybeImprecise;
use rand::prelude::SliceRandom;
use rich_text::RichString;
use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::search::QueryRestriction;

/// Data structure for Cards. All fields are mandatory.
#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq, Default)]
pub struct Card {
    /// A value that uniquely identifies the card. This is necessary because many cards may have the same name.
    pub id: String,
    /// The card's name.
    pub name: String,
    /// Image names that the card may use. If this is empty, the name is used to generate an image name.
    #[serde(default)]
    pub img: Vec<String>,
    /// The card's text, excluding cost, stats and typeline.
    pub description: RichString,
    /// The card's blood cost.
    pub cost: MaybeImprecise,
    /// The card's health.
    pub health: MaybeImprecise,
    /// The card's overkill protection.
    pub defense: MaybeImprecise,
    /// The card's power.
    pub power: MaybeImprecise,
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
    #[serde(default)]
    /// The card's flavor text
    pub flavor_text: String,
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

impl Read for Card {
    fn get_flavor_text(&self) -> Option<&str> {
        Some(&self.flavor_text)
    }
    /// Return a card's numeric property, if it has it.
    /// Will only return None if the card's type contains the word "command" and the given value is not `NumberProperties::Cost`.
    fn get_num_property(&self, property: &Number) -> Option<MaybeImprecise> {
        match property {
            Number::Cost => Some(self.cost.clone()),
            Number::Health => {
                if self.r#type.contains("command") {
                    None
                } else {
                    Some(self.health.clone())
                }
            }
            Number::Defense => {
                if self.r#type.contains("command") {
                    None
                } else {
                    Some(self.defense.clone())
                }
            }
            Number::Power => {
                if self.r#type.contains("command") {
                    None
                } else {
                    Some(self.power.clone())
                }
            }
        }
    }

    /// Return a card's text property, if it has it.
    /// Always returns Some.
    fn get_text_property(&self, property: &Text) -> Option<String> {
        Some(match property {
            Text::Id => self.id.to_string(),
            Text::Name => self.name.to_string(),
            Text::Type => self.r#type.to_string(),
            Text::Description => self.description.to_string(),
            Text::FlavorText => self.flavor_text.to_string(),
        })
    }

    /// Return a card's array property, if it has it.
    /// Always returns Some.
    fn get_vec_property(&self, property: &Array) -> Option<&[String]> {
        Some(match property {
            Array::Functions => &self.functions,
            Array::Kins => &self.kins,
            Array::Artists => &self.artists,
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
    fn get_description(&self) -> Option<&RichString> {
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

impl Read for &Card {
    fn get_flavor_text(&self) -> Option<&str> {
        Some(&self.flavor_text)
    }
    /// Return a card's numeric property, if it has it.
    /// Will only return None if the card's type contains the word "command" and the given value is not `NumberProperties::Cost`.
    fn get_num_property(&self, property: &Number) -> Option<MaybeImprecise> {
        match property {
            Number::Cost => Some(self.cost.clone()),
            Number::Health => {
                if self.r#type.contains("command") {
                    None
                } else {
                    Some(self.health.clone())
                }
            }
            Number::Defense => {
                if self.r#type.contains("command") {
                    None
                } else {
                    Some(self.defense.clone())
                }
            }
            Number::Power => {
                if self.r#type.contains("command") {
                    None
                } else {
                    Some(self.power.clone())
                }
            }
        }
    }

    /// Return a card's text property, if it has it.
    /// Always returns Some.
    fn get_text_property(&self, property: &Text) -> Option<String> {
        Some(match property {
            Text::Id => self.id.to_string(),
            Text::Name => self.name.to_string(),
            Text::Type => self.r#type.to_string(),
            Text::Description => self.description.to_string(),
            Text::FlavorText => self.flavor_text.to_string(),
        })
    }

    /// Return a card's array property, if it has it.
    /// Always returns Some.
    fn get_vec_property(&self, property: &Array) -> Option<&[String]> {
        Some(match property {
            Array::Functions => &self.functions,
            Array::Kins => &self.kins,
            Array::Artists => &self.artists,
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
    fn get_description(&self) -> Option<&RichString> {
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

impl Read for CardId {
    fn get_flavor_text(&self) -> Option<&str> {
        None
    }
    fn get_num_property(&self, property: &Number) -> Option<MaybeImprecise> {
        match property {
            Number::Cost => self.cost.clone(),
            Number::Health => {
                if self.r#type.as_ref().is_some_and(|x| x.contains("command")) {
                    None
                } else {
                    self.health.clone()
                }
            }
            Number::Defense => {
                if self.r#type.as_ref().is_some_and(|x| x.contains("command")) {
                    None
                } else {
                    self.defense.clone()
                }
            }
            Number::Power => {
                if self.r#type.as_ref().is_some_and(|x| x.contains("command")) {
                    None
                } else {
                    self.power.clone()
                }
            }
        }
    }

    fn get_text_property(&self, property: &Text) -> Option<String> {
        match property {
            Text::Name => self.name.as_deref().map(ToString::to_string),
            Text::Type => self.r#type.as_deref().map(ToString::to_string),
            Text::Description => self.description.as_ref().map(ToString::to_string),
            Text::FlavorText | Text::Id => None,
        }
    }

    fn get_vec_property(&self, property: &Array) -> Option<&[String]> {
        match property {
            Array::Functions => self.functions.as_deref(),
            Array::Kins => self.kins.as_deref(),
            Array::Artists => None,
        }
    }

    fn get_keywords(&self) -> Option<&[Keyword]> {
        self.keywords.as_deref()
    }

    fn get_name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn get_description(&self) -> Option<&RichString> {
        self.description.as_ref()
    }

    fn get_type(&self) -> Option<&str> {
        self.r#type.as_deref()
    }

    fn get_kins(&self) -> Option<&[String]> {
        self.kins.as_deref()
    }
}

impl Read for &CardId {
    fn get_flavor_text(&self) -> Option<&str> {
        None
    }
    fn get_num_property(&self, property: &Number) -> Option<MaybeImprecise> {
        match property {
            Number::Cost => self.cost.clone(),
            Number::Health => {
                if self.r#type.as_ref().is_some_and(|x| x.contains("command")) {
                    None
                } else {
                    self.health.clone()
                }
            }
            Number::Defense => {
                if self.r#type.as_ref().is_some_and(|x| x.contains("command")) {
                    None
                } else {
                    self.defense.clone()
                }
            }
            Number::Power => {
                if self.r#type.as_ref().is_some_and(|x| x.contains("command")) {
                    None
                } else {
                    self.power.clone()
                }
            }
        }
    }

    fn get_text_property(&self, property: &Text) -> Option<String> {
        match property {
            Text::Name => self.name.as_deref().map(ToString::to_string),
            Text::Type => self.r#type.as_deref().map(ToString::to_string),
            Text::Description => self.description.as_ref().map(ToString::to_string),
            Text::FlavorText | Text::Id => None,
        }
    }

    fn get_vec_property(&self, property: &Array) -> Option<&[String]> {
        match property {
            Array::Functions => self.functions.as_deref(),
            Array::Kins => self.kins.as_deref(),
            Array::Artists => None,
        }
    }

    fn get_keywords(&self) -> Option<&[Keyword]> {
        self.keywords.as_deref()
    }

    fn get_name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn get_description(&self) -> Option<&RichString> {
        self.description.as_ref()
    }

    fn get_type(&self) -> Option<&str> {
        self.r#type.as_deref()
    }

    fn get_kins(&self) -> Option<&[String]> {
        self.kins.as_deref()
    }
}

/// Data structure for card identities. These card identities are slightly more general than the concept within the game, as they allow you to match things that are only relevant for searching cards.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct CardId {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<MaybeImprecise>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<RichString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub keywords: Option<Vec<Keyword>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub kins: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<MaybeImprecise>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defense: Option<MaybeImprecise>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power: Option<MaybeImprecise>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub abilities: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub functions: Option<Vec<String>>,
}

/// A keyword may contain data. This data may be a string or a `CardID`.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum KeywordData {
    CardId(CardId),
    String(String),
}

/// A card's Keyword.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Keyword {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<KeywordData>,
}

impl Card {
    /// Obtains a randomly selected image name from the `Card`'s img field. If it can't, it gets an image name based on its name.
    #[must_use]
    pub fn get_image(&self) -> String {
        self.img
            .choose(&mut rand::thread_rng())
            .cloned()
            .unwrap_or_else(|| self.name.replace(' ', ""))
    }
}

impl CardId {
    #[must_use]
    /// Creates a vector of `QueryRestriction`s defined by the `CardID`.
    pub fn get_as_query(&self) -> Vec<QueryRestriction> {
        let mut restrictions = vec![];

        if let Some(name) = &self.name {
            restrictions.push(QueryRestriction::Contains(Text::Name, name.clone()));
        }

        if let Some(kins) = &self.kins {
            for kin in kins {
                restrictions.push(QueryRestriction::Has(Array::Kins, kin.clone()));
            }
        }

        if let Some(keywords) = &self.keywords {
            for keyword in keywords {
                restrictions.push(QueryRestriction::HasKw(keyword.name.clone()));
            }
        }

        if let Some(cost) = &self.cost {
            restrictions.push(QueryRestriction::Comparison(
                Number::Cost,
                cost.as_comparison(),
            ));
        }

        if let Some(health) = &self.health {
            restrictions.push(QueryRestriction::Comparison(
                Number::Health,
                health.as_comparison(),
            ));
        }

        if let Some(power) = &self.power {
            restrictions.push(QueryRestriction::Comparison(
                Number::Power,
                power.as_comparison(),
            ));
        }

        if let Some(defense) = &self.defense {
            restrictions.push(QueryRestriction::Comparison(
                Number::Defense,
                defense.as_comparison(),
            ));
        }

        restrictions
    }
}
