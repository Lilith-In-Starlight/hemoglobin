pub mod fuzzy;
pub mod query_parser;
use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Display, Write},
};

use fuzzy::weighted_compare;
use regex::Regex;

use crate::{
    cards::{
        properties::{Array, Number, Read, Text},
        Keyword, KeywordData,
    },
    numbers::{Comparison, ImpreciseOrd},
    QueryMatch,
};

/// Errors that might happen during searching
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

impl From<bool> for QueryMatch {
    fn from(value: bool) -> Self {
        if value {
            Self::Match
        } else {
            Self::NotMatch
        }
    }
}

/// Represents a search query
#[derive(Debug, Clone)]
pub struct Query {
    pub name: String,
    pub restrictions: Vec<QueryRestriction>,
    pub sort: Sort,
}

impl Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut text_properties = vec![];
        for restriction in &self.restrictions {
            text_properties.push(format!("{restriction}"));
        }
        match &self.sort {
            Sort::None => (),
            Sort::Fuzzy if !self.name.is_empty() => {
                text_properties.push("sorted by fuzzy match".to_string());
            }
            Sort::Fuzzy => text_properties.push("sorted by Name in Ascending order".to_owned()),
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
            QueryRestriction::DevouredBy(devourer) => {
                write!(f, "which are devoured by [{devourer}]")
            }
            QueryRestriction::Fuzzy(text) => write!(f, "with \"{text}\" written on them"),
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
                Array::Functions => write!(f, "which can be used to \"{text}\""),
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

/// Represents a specific restriction that a `Query` will apply to cards.
#[derive(Debug, Clone)]
pub enum QueryRestriction {
    Fuzzy(String),
    Devours(Query),
    DevouredBy(Query),
    Comparison(Number, Comparison),
    Contains(Text, String),
    Regex(Text, Regex),
    Has(Array, String),
    HasKw(String),
    Not(Query),
    LenientNot(Query),
    Group(Query),
    Or(Query, Query),
    Xor(Query, Query),
}

/// Represents a specific ordering for sorting.
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

/// Specific ways to sort cards.
#[derive(Debug, Clone, Copy)]
pub enum Sort {
    /// Do not sort
    None,
    /// Sort by how much they match a string
    Fuzzy,
    Alphabet(Text, Ordering),
    Numeric(Number, Ordering),
}

/// Restriction that matches only if a card contains some text
#[must_use]
pub fn fuzzy(card: &impl Read, query: &str) -> bool {
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

/// The Cache for `devouredby` queries.
pub type Cache<T> = RefCell<HashMap<String, Vec<T>>>;

/// Function that takes `cards` and outputs a vector pointing to all the cards that matched the `query`.
#[must_use]
pub fn search<'a, 'b, C, I>(query: &Query, cards: I) -> Vec<&'a C>
where
    C: Read + Clone + 'a,
    I: IntoIterator<Item = &'a C> + Clone + 'b,
    &'a C: Read,
{
    let cards_clone = cards.clone();
    let cache = Cache::new(HashMap::new());
    let mut results: Vec<&C> = cards
        .into_iter()
        .filter(|card| matches_query(card, query, &cards_clone, &cache) == QueryMatch::Match)
        .collect();

    match &query.sort {
        Sort::None => (),
        Sort::Fuzzy if !query.name.is_empty() => results.sort_by(|a, b| {
            weighted_compare(b, &query.name)
                .partial_cmp(&weighted_compare(a, &query.name))
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        Sort::Fuzzy => results.sort_by(|a, b| Ord::cmp(&a.get_name(), &b.get_name())),
        Sort::Alphabet(property, Ordering::Ascending) => results.sort_by(|a, b| {
            Ord::cmp(
                &a.get_text_property(property),
                &b.get_text_property(property),
            )
        }),
        Sort::Numeric(property, Ordering::Ascending) => results.sort_by(|a, b| {
            ImpreciseOrd::imprecise_cmp(
                &a.get_num_property(property),
                &b.get_num_property(property),
            )
        }),
        Sort::Alphabet(property, Ordering::Descending) => {
            results.sort_by(|a, b| {
                Ord::cmp(
                    &a.get_text_property(property),
                    &b.get_text_property(property),
                )
                .reverse()
            });
        }
        Sort::Numeric(property, Ordering::Descending) => results.sort_by(|a, b| {
            ImpreciseOrd::imprecise_cmp(
                &a.get_num_property(property),
                &b.get_num_property(property),
            )
            .reverse()
        }),
    }

    results
}

/// This function checks whether a `card` matches a specific `query`'s restrictions. Since `devouredby` queries always require two searches, the results of the first search are stored in a `cache` that is internally mutable. This cache is only ever mutated the first time a devouredby query is executed.
/// The sum total of available `cards` is passed in order to perform searches. This function clones these cards, so this value should be an Iterator.
#[allow(clippy::too_many_lines)]
pub fn matches_query<'a, 'b, C, T, I>(
    card: &C,
    query: &Query,
    cards: &I,
    cache: &Cache<&'a T>,
) -> QueryMatch
where
    C: Read,
    T: Read + 'a + Clone,
    &'a T: Read,
    I: IntoIterator<Item = &'a T> + Clone,
{
    let mut filtered = QueryMatch::Match;
    for res in &query.restrictions {
        match res {
            QueryRestriction::Regex(property, regex) => {
                let matches = {
                    match card.get_text_property(property) {
                        None => QueryMatch::NotHave,
                        Some(value) => {
                            if regex.is_match(&value.to_lowercase()) {
                                QueryMatch::Match
                            } else {
                                QueryMatch::NotMatch
                            }
                        }
                    }
                };
                filtered = filtered.and(matches);
            }
            QueryRestriction::Xor(group1, group2) => {
                let res1 = matches_query(card, group1, cards, cache);
                let res2 = matches_query(card, group2, cards, cache);
                filtered = filtered.and(res1.xor(res2));
            }
            QueryRestriction::Or(group1, group2) => {
                filtered = filtered.and(
                    matches_query(card, group1, cards, cache)
                        .or(matches_query(card, group2, cards, cache)),
                );
            }
            QueryRestriction::Group(group) => {
                filtered = filtered.and(matches_query(card, group, cards, cache));
            }
            QueryRestriction::Fuzzy(x) => {
                filtered = filtered.and(if fuzzy(card, x) {
                    QueryMatch::Match
                } else {
                    QueryMatch::NotMatch
                });
            }
            QueryRestriction::Comparison(field, comparison) => {
                if let Some(name) = card.get_name() {
                    if name == "Crypt Mantis" {
                        println!(
                            "{:#?}",
                            card.get_num_property(field)
                                .unwrap()
                                .imprecise_cmp(comparison)
                        );
                    }
                }
                filtered = filtered.and(comparison.compare(&card.get_num_property(field)));
            }
            QueryRestriction::Contains(field, contains) => {
                let matches = match card.get_text_property(field) {
                    Some(property) => {
                        if property.to_lowercase().contains(&contains.to_lowercase()) {
                            QueryMatch::Match
                        } else {
                            QueryMatch::NotMatch
                        }
                    }
                    None => QueryMatch::NotHave,
                };
                filtered = filtered.and(matches);
            }
            QueryRestriction::Has(field, thing) => {
                let matches = match_in_vec(card.get_vec_property(field), |text| {
                    text.to_lowercase().contains(&thing.to_lowercase())
                });
                filtered = filtered.and(matches);
            }
            QueryRestriction::HasKw(thing) => {
                let matches = match_in_vec(card.get_keywords(), |keyword| {
                    keyword.name.to_lowercase().contains(&thing.to_lowercase())
                });
                filtered = filtered.and(matches);
            }
            QueryRestriction::Not(queryres) => {
                filtered = filtered.and(!matches_query(card, queryres, cards, cache));
            }
            QueryRestriction::LenientNot(queryres) => {
                filtered = filtered.and(
                    if matches_query(card, queryres, cards, cache) == QueryMatch::Match {
                        QueryMatch::NotMatch
                    } else {
                        QueryMatch::Match
                    },
                );
            }
            QueryRestriction::Devours(query) => {
                let matches = match_in_vec(card.get_keywords(), |keyword| {
                    if keyword.name == "devours" {
                        if let Some(KeywordData::CardID(ref devoured_id)) = keyword.data {
                            matches_query(&devoured_id, query, cards, cache) == QueryMatch::Match
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                });
                filtered = filtered.and(matches);
            }
            QueryRestriction::DevouredBy(devoured_by) => {
                let key = format!("{devoured_by}");
                let devoured_cards;
                let maybe_devourees = RefCell::borrow(cache).get(&key).cloned();
                if let Some(value) = maybe_devourees {
                    devoured_cards = value;
                } else {
                    let cloned_cards = cards.clone();
                    let devourers: Vec<&T> = cards
                        .clone()
                        .into_iter()
                        .filter(|card| {
                            matches_query(card, devoured_by, &cloned_cards, cache)
                                == QueryMatch::Match
                        })
                        .collect();

                    let mut queries: Vec<Query> = vec![];

                    for devourer in devourers {
                        if let Some(Keyword {
                            name: _,
                            data: Some(KeywordData::CardID(card_id)),
                        }) = devourer
                            .get_keywords()
                            .and_then(|x| x.iter().find(|x| x.name == "devours"))
                        {
                            queries.push(Query {
                                name: String::new(),
                                restrictions: card_id.get_as_query(),
                                sort: Sort::None,
                            });
                        }
                    }

                    let devourees_query = queries
                        .into_iter()
                        .reduce(|first, second| Query {
                            name: String::new(),
                            restrictions: vec![QueryRestriction::Or(first, second)],
                            sort: Sort::None,
                        })
                        .unwrap_or(Query {
                            name: String::new(),
                            restrictions: vec![],
                            sort: Sort::None,
                        });

                    let devourees_query = Query {
                        name: query.name.clone(),
                        restrictions: devourees_query.restrictions,
                        sort: query.sort,
                    };

                    devoured_cards = search(&devourees_query, cloned_cards);
                    cache.borrow_mut().insert(key, devoured_cards.clone());
                }
                if devoured_cards
                    .iter()
                    .any(|x| x.get_name() == card.get_name())
                {
                    filtered = filtered.and(QueryMatch::Match);
                } else {
                    filtered = filtered.and(QueryMatch::NotMatch);
                }
            }
        }
    }
    filtered
}

/// Returns whether any part of an optional `vec` fulfills a `cond`ition.
pub fn match_in_vec<T>(vec: Option<&[T]>, cond: impl Fn(&T) -> bool) -> QueryMatch {
    match vec {
        Some(vec) => {
            if vec.iter().any(cond) {
                QueryMatch::Match
            } else {
                QueryMatch::NotMatch
            }
        }
        None => QueryMatch::NotHave,
    }
}
