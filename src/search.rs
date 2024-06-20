pub mod query_parser;
use std::fmt::{Display, Write};

use regex::Regex;
use serde::Deserialize;

use crate::{
    cards::{ArrayProperties, NumberProperties, ReadProperties, StringProperties},
    QueryMatch,
};

#[derive(Debug)]
pub enum Errors {
    NonRegexable(String),
    InvalidOr,
    InvalidComparisonString,
    UnknownSubQueryParam(String),
    UnknownStringParam(String),
    InvalidOrdering(String),
    InvalidPolarity,
    NotSortable,
    UnclosedSubquery,
    UnclosedString,
    UnclosedRegex,
    RegexErr(regex::Error),
    AttemptedEmptyParamName,
}

#[derive(Debug, Clone)]
pub struct Query {
    pub name: String,
    pub devoured_by: Option<Box<Query>>,
    pub restrictions: Vec<QueryRestriction>,
    pub sort: Sort,
}

impl Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut text_properties = vec![];
        match &self.devoured_by {
            Some(devoured_by) => text_properties.push(format!("devoured by [{devoured_by}]")),
            None => (),
        }
        for restriction in &self.restrictions {
            text_properties.push(format!("{restriction}"));
        }
        match &self.sort {
            Sort::None => (),
            Sort::Fuzzy => text_properties.push("sorted by fuzzy match".to_string()),
            Sort::Alphabet(property, order) => {
                text_properties.push(format!("sorted by {property} in {order} order"));
            }
            Sort::Numeric(property, order) => {
                text_properties.push(format!("sorted by {property} in {order} order"));
            }
        }

        let text_properties = text_properties.into_iter().reduce(|mut acc, el| {
            write!(&mut acc, ", {el}").unwrap();
            acc
        });
        match text_properties {
            Some(text_properties) => write!(f, "Cards {text_properties}"),
            None => write!(f, "Cards"),
        }
    }
}

#[allow(clippy::match_wildcard_for_single_variants)]
impl Display for QueryRestriction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryRestriction::Fuzzy(text) => write!(f, "with {text} written on them"),
            QueryRestriction::Devours(devourees) => write!(f, "that devour [{devourees}]"),
            QueryRestriction::Comparison(property, comparison) => {
                write!(f, "with {property} {comparison}")
            }
            QueryRestriction::Contains(property, text) => {
                write!(f, "whose {property} contains \"{text}\"")
            }
            QueryRestriction::Regex(property, regex) => {
                write!(f, "whose {property} matches /{regex}/")
            }
            QueryRestriction::Has(property, text) => match property {
                ArrayProperties::Functions => write!(f, "which can be used to \"{text}\""),
                property => write!(f, "whose {property} have \"{text}\" among them"),
            },
            QueryRestriction::HasKw(keyword) => write!(f, "with a \"{keyword}\" keyword"),
            QueryRestriction::Not(query) => write!(f, "that aren't [{query}]"),
            QueryRestriction::LenientNot(query) => write!(
                f,
                "that aren't [{query}], counting lacks of a property as a non-match"
            ),
            QueryRestriction::Group(query) => write!(f, "which are [{query}]"),
            QueryRestriction::Or(a, b) => write!(f, "which are [{a}] or [{b}]"),
            QueryRestriction::Xor(a, b) => write!(f, "which are [{a}] xor [{b}] but not both"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum QueryRestriction {
    Fuzzy(String),
    Devours(Query),
    Comparison(NumberProperties, Comparison),
    Contains(StringProperties, String),
    Regex(StringProperties, Regex),
    Has(ArrayProperties, String),
    HasKw(String),
    Not(Query),
    LenientNot(Query),
    Group(Query),
    Or(Query, Query),
    Xor(Query, Query),
}

#[derive(Debug, Clone, Copy)]
pub enum Ordering {
    Ascending,
    Descending,
}

impl Display for Ordering {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ascending => write!(f, "Ascending"),
            Self::Descending => write!(f, "Descending"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Sort {
    None,
    Fuzzy,
    Alphabet(StringProperties, Ordering),
    Numeric(NumberProperties, Ordering),
}

#[derive(Deserialize)]
pub struct QueryParams {
    pub query: Option<String>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum Comparison {
    GreaterThan(usize),
    GreaterThanOrEqual(usize),
    LowerThanOrEqual(usize),
    Equal(usize),
    LowerThan(usize),
    NotEqual(usize),
}

impl Display for Comparison {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GreaterThan(number) => write!(f, "greater than {number}"),
            Self::GreaterThanOrEqual(number) => write!(f, "greater than or equal to {number}"),
            Self::LowerThanOrEqual(number) => write!(f, "lower than than or equal to {number}"),
            Self::Equal(number) => write!(f, "equal to {number}"),
            Self::LowerThan(number) => write!(f, "lower than {number}"),
            Self::NotEqual(number) => write!(f, "other than {number}"),
        }
    }
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
