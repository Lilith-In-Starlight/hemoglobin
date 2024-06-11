pub mod query_parser;
use serde::Deserialize;

use crate::cards::{Card, Keyword};

#[derive(Deserialize)]
pub struct QueryParams {
    pub query: Option<String>,
}

#[derive(Deserialize)]
pub enum Comparison {
    GreaterThan(usize),
    GreaterThanOrEqual(usize),
    LowerThanOrEqual(usize),
    Equal(usize),
    LowerThan(usize),
    NotEqual(usize),
}

impl Comparison {
    pub fn compare<T: PartialOrd<usize>>(&self, a: &T) -> bool {
        match self {
            Comparison::GreaterThan(x) => a > x,
            Comparison::Equal(x) => a == x,
            Comparison::LowerThan(x) => a < x,
            Comparison::NotEqual(x) => a != x,
            Comparison::GreaterThanOrEqual(x) => a >= x,
            Comparison::LowerThanOrEqual(x) => a <= x,
        }
    }
}

pub enum Errors {
    InvalidComparisonString,
    UnknownParam,
}

#[must_use]
pub fn fuzzy(card: &Card, query: &str) -> bool {
    card.description
        .to_lowercase()
        .contains(&query.to_lowercase())
        || card.name.to_lowercase().contains(&query.to_lowercase())
        || card.r#type.to_lowercase().contains(&query.to_lowercase())
        || card.kins.iter().any(|x| x.contains(&query.to_lowercase()))
        || card
            .keywords
            .iter()
            .any(|x| x.name.contains(&query.to_lowercase()))
}

pub enum QueryRestriction {
    Fuzzy(String),
    Comparison(Box<dyn Fn(&Card) -> usize>, Comparison),
    Contains(Box<dyn Fn(&Card) -> &str>, String),
    Has(Box<dyn Fn(&Card) -> &[String]>, String),
    HasKw(Box<dyn Fn(&Card) -> &[Keyword]>, String),
}
