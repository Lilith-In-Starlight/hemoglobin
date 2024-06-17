pub mod query_parser;
use serde::Deserialize;

use crate::{
    cards::{ArrayProperties, NumberProperties, ReadProperties, StringProperties},
    QueryMatch,
};

#[derive(Debug)]
pub struct Query {
    pub name: String,
    pub restrictions: Vec<QueryRestriction>,
    pub sort: Sort,
}

#[derive(Debug)]
pub enum Ordering {
    Ascending,
    Descending,
}

#[derive(Debug)]
pub enum Sort {
    Fuzzy,
    Alphabet(StringProperties, Ordering),
    Numeric(NumberProperties, Ordering),
}

#[derive(Deserialize)]
pub struct QueryParams {
    pub query: Option<String>,
}

#[derive(Deserialize, Debug)]
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

    pub fn maybe_compare<T: PartialOrd<usize>>(&self, a: Option<T>) -> QueryMatch {
        match a {
            Some(a) => {
                if self.compare(&a) {
                    QueryMatch::Match
                } else {
                    QueryMatch::NotMatch
                }
            }
            None => QueryMatch::NotHave,
        }
    }
}

#[derive(Debug)]
pub enum Errors {
    InvalidOr,
    InvalidComparisonString,
    UnknownSubQueryParam(String),
    UnknownStringParam(String),
    InvalidOrdering(String),
    InvalidPolarity,
}

#[must_use]
pub fn fuzzy(card: &impl ReadProperties, query: &str) -> bool {
    card.get_description()
        .is_some_and(|x| x.to_lowercase().contains(&query.to_lowercase()))
        || card
            .get_name()
            .is_some_and(|x| x.to_lowercase().contains(&query.to_lowercase()))
        || card
            .get_type()
            .is_some_and(|x| x.to_lowercase().contains(&query.to_lowercase()))
        || card
            .get_kins()
            .is_some_and(|x| x.iter().any(|x| x.contains(&query.to_lowercase())))
        || card
            .get_keywords()
            .is_some_and(|x| x.iter().any(|x| x.name.contains(&query.to_lowercase())))
}

#[derive(Debug)]
pub enum QueryRestriction {
    Fuzzy(String),
    Devours(Query),
    Comparison(NumberProperties, Comparison),
    Contains(StringProperties, String),
    Has(ArrayProperties, String),
    HasKw(String),
    Not(Query),
    LenientNot(Query),
    Group(Query),
    Or(Query, Query),
    Xor(Query, Query),
}
