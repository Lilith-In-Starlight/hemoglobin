use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct Card {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub img: Vec<String>,
    pub description: String,
    pub cost: usize,
    pub health: usize,
    pub defense: usize,
    pub power: usize,
    pub r#type: String,
    #[serde(default)]
    pub keywords: Vec<Keyword>,
    #[serde(default)]
    pub kins: Vec<String>,
    #[serde(default)]
    pub abilities: Vec<String>,
    #[serde(default)]
    pub artists: Vec<String>,
    pub set: String,
    pub legality: HashMap<String, String>,
    #[serde(default)]
    pub other: Vec<String>,
    #[serde(default)]
    pub functions: Vec<String>,
}

impl Card {
    #[must_use]
    pub fn get_cost(&self) -> usize {
        self.cost
    }
    #[must_use]
    pub fn get_name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub fn get_type(&self) -> &str {
        &self.r#type
    }
    #[must_use]
    pub fn get_kins(&self) -> &[String] {
        &self.kins
    }
    #[must_use]
    pub fn get_keywords(&self) -> &[Keyword] {
        &self.keywords
    }
    #[must_use]
    pub fn get_health(&self) -> usize {
        self.health
    }
    #[must_use]
    pub fn get_power(&self) -> usize {
        self.power
    }
    #[must_use]
    pub fn get_defense(&self) -> usize {
        self.defense
    }
    #[must_use]
    pub fn get_abilities(&self) -> &[String] {
        &self.abilities
    }
    #[must_use]
    pub fn get_artists(&self) -> &[String] {
        &self.artists
    }
    #[must_use]
    pub fn get_set(&self) -> &str {
        &self.set
    }
    #[must_use]
    pub fn get_description(&self) -> &str {
        &self.description
    }
    #[must_use]
    pub fn get_legality(&self) -> &HashMap<String, String> {
        &self.legality
    }
    #[must_use]
    pub fn get_functions(&self) -> &[String] {
        &self.functions
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct CardID {
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

impl CardID {
    #[must_use]
    pub fn get_cost(&self) -> Option<usize> {
        self.cost
    }
    #[must_use]
    pub fn get_name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    #[must_use]
    pub fn get_type(&self) -> Option<&str> {
        self.r#type.as_deref()
    }
    #[must_use]
    pub fn get_kins(&self) -> Option<&[String]> {
        self.kins.as_deref()
    }
    #[must_use]
    pub fn get_keywords(&self) -> Option<&[Keyword]> {
        self.keywords.as_deref()
    }
    #[must_use]
    pub fn get_health(&self) -> Option<usize> {
        self.health
    }
    #[must_use]
    pub fn get_power(&self) -> Option<usize> {
        self.power
    }
    #[must_use]
    pub fn get_defense(&self) -> Option<usize> {
        self.defense
    }
    #[must_use]
    pub fn get_abilities(&self) -> Option<&[String]> {
        self.abilities.as_deref()
    }
    #[must_use]
    pub fn get_description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    #[must_use]
    pub fn get_functions(&self) -> Option<&[String]> {
        self.functions.as_deref()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum KeywordData {
    CardID(CardID),
    String(String),
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Keyword {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<KeywordData>,
}
