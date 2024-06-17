use std::{collections::HashMap, fmt::Display};

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

pub trait ReadProperties {
    fn get_num_property(&self, property: &NumberProperties) -> Option<usize>;
    fn get_str_property(&self, property: &StringProperties) -> Option<&str>;
    fn get_vec_property(&self, property: &ArrayProperties) -> Option<&[String]>;
    fn get_keywords(&self) -> Option<&[Keyword]>;
    fn get_name(&self) -> Option<&str>;
    fn get_description(&self) -> Option<&str>;
    fn get_type(&self) -> Option<&str>;
    fn get_kins(&self) -> Option<&[String]>;
}

impl ReadProperties for Card {
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

    fn get_str_property(&self, property: &StringProperties) -> Option<&str> {
        Some(match property {
            StringProperties::Name => &self.name,
            StringProperties::Type => &self.r#type,
            StringProperties::Description => &self.description,
        })
    }

    fn get_vec_property(&self, property: &ArrayProperties) -> Option<&[String]> {
        Some(match property {
            ArrayProperties::Functions => &self.functions,
            ArrayProperties::Kins => &self.kins,
        })
    }

    fn get_keywords(&self) -> Option<&[Keyword]> {
        Some(&self.keywords)
    }

    fn get_name(&self) -> Option<&str> {
        Some(&self.name)
    }

    fn get_description(&self) -> Option<&str> {
        Some(&self.description)
    }

    fn get_type(&self) -> Option<&str> {
        Some(&self.r#type)
    }

    fn get_kins(&self) -> Option<&[String]> {
        Some(&self.kins)
    }
}

impl ReadProperties for &Card {
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

    fn get_str_property(&self, property: &StringProperties) -> Option<&str> {
        Some(match property {
            StringProperties::Name => &self.name,
            StringProperties::Type => &self.r#type,
            StringProperties::Description => &self.description,
        })
    }

    fn get_vec_property(&self, property: &ArrayProperties) -> Option<&[String]> {
        Some(match property {
            ArrayProperties::Functions => &self.functions,
            ArrayProperties::Kins => &self.kins,
        })
    }

    fn get_keywords(&self) -> Option<&[Keyword]> {
        Some(&self.keywords)
    }

    fn get_name(&self) -> Option<&str> {
        Some(&self.name)
    }

    fn get_description(&self) -> Option<&str> {
        Some(&self.description)
    }

    fn get_type(&self) -> Option<&str> {
        Some(&self.r#type)
    }

    fn get_kins(&self) -> Option<&[String]> {
        Some(&self.kins)
    }
}

impl ReadProperties for CardID {
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
            StringProperties::Name => self.name.as_deref(),
            StringProperties::Type => self.r#type.as_deref(),
            StringProperties::Description => self.description.as_deref(),
        }
    }

    fn get_vec_property(&self, property: &ArrayProperties) -> Option<&[String]> {
        match property {
            ArrayProperties::Functions => self.functions.as_deref(),
            ArrayProperties::Kins => self.kins.as_deref(),
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

impl ReadProperties for &CardID {
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
            StringProperties::Name => self.name.as_deref(),
            StringProperties::Type => self.r#type.as_deref(),
            StringProperties::Description => self.description.as_deref(),
        }
    }

    fn get_vec_property(&self, property: &ArrayProperties) -> Option<&[String]> {
        match property {
            ArrayProperties::Functions => self.functions.as_deref(),
            ArrayProperties::Kins => self.kins.as_deref(),
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

#[derive(Debug)]
pub enum NumberProperties {
    Cost,
    Health,
    Power,
    Defense,
}

#[derive(Debug)]
pub enum ArrayProperties {
    Functions,
    Kins,
}

#[derive(Debug)]
pub enum StringProperties {
    Name,
    Type,
    Description,
}
