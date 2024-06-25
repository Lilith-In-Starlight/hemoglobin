#![warn(clippy::pedantic)]

//! Hemolymph is a library containing data structures and functions useful for the card game Bloodless. It is used by [Hemolymph](http://hemolymph.net), the official card search engine.
//!
//! The two datastructures are `Card`, which represents a card and `CardID`, which represents a card identity. Card identities in this library do not represent card identities as defined by the game's rules, but rather as a more general structure for identifying cards.
//!
//! This library contains the search functions used by Hemolymph.

use std::{
    cell::RefCell,
    cmp::{max, min, Ordering},
    collections::HashMap,
    ops::Not,
};

use cards::{Keyword, KeywordData, ReadProperties};
use rust_fuzzy_search::fuzzy_compare;
use search::{Query, QueryRestriction, Sort};

pub mod cards;
pub mod search;

/// Represents whether a query has been matched or not. This is not always a boolean value, but instead a ternary value, as cards may have undefined properties.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum QueryMatch {
    /// Card did not have the requested property.
    NotHave,
    /// Card had the requested property and it did not matched the requested value.
    NotMatch,
    /// Card had the requested property and it matched the requested value.
    Match,
}

impl QueryMatch {
    /// A ternary OR which outputs the highest-valued result between `self` and `b`, where a `Match` is considered highest and `NotHave` is considered lowest.
    #[must_use]
    pub fn or(self, b: Self) -> Self {
        max(self, b)
    }
    /// A ternary XOR which outputs the highest-valued result between `self` and `b`, if they are not equal.
    /// If both values are `Match` or `NotMatch`, the output will be `NotMatch`.
    /// If both values are `NotHave`, the output will be `NotHave`.
    /// If no value is Match and there is a `NotHave`, the output will be `NotHave`.
    #[must_use]
    pub fn xor(self, b: Self) -> Self {
        match (self, b) {
            (Self::Match, Self::Match) => Self::NotMatch,
            (Self::NotHave, Self::NotHave) => Self::NotHave,
            (Self::Match, Self::NotMatch | Self::NotHave)
            | (Self::NotMatch | Self::NotHave, Self::Match) => Self::Match,
            (Self::NotMatch | Self::NotHave, Self::NotMatch) | (Self::NotMatch, Self::NotHave) => {
                Self::NotMatch
            }
        }
    }
    /// A ternary AND which outputs the lowest-valued result between `self` and `b`, where a `Match` is considered highest and `NotHave` is considered lowest.
    #[must_use]
    pub fn and(self, b: Self) -> Self {
        min(self, b)
    }
}

impl Not for QueryMatch {
    type Output = QueryMatch;

    /// Ternary NOT where `NotHave` is considered opposite to itself.
    fn not(self) -> Self::Output {
        match self {
            Self::Match => Self::NotMatch,
            Self::NotMatch => Self::Match,
            Self::NotHave => Self::NotHave,
        }
    }
}

/// The Cache for `devouredby` queries.
type Cache<T> = RefCell<HashMap<String, Vec<T>>>;

/// This function checks whether a `card` matches a specific `query`'s restrictions. Since `devouredby` queries always require two searches, the results of the first search are stored in a `cache` that is internally mutable. This cache is only ever mutated the first time a devouredby query is executed.
/// The sum total of available `cards` is passed in order to perform searches. This function clones these cards, so this value should be an Iterator.
#[allow(clippy::too_many_lines)]
fn apply_restriction<'a, 'b, C, T, I>(
    card: &C,
    query: &Query,
    cards: &I,
    cache: &Cache<&'a T>,
) -> QueryMatch
where
    C: ReadProperties,
    T: ReadProperties + 'a + Clone,
    &'a T: ReadProperties,
    I: IntoIterator<Item = &'a T> + Clone,
{
    let mut filtered = QueryMatch::Match;
    for res in &query.restrictions {
        match res {
            QueryRestriction::Regex(property, regex) => {
                let matches = {
                    match card.get_str_property(property) {
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
                let res1 = apply_restriction(card, group1, cards, cache);
                let res2 = apply_restriction(card, group2, cards, cache);
                filtered = filtered.and(res1.xor(res2));
            }
            QueryRestriction::Or(group1, group2) => {
                filtered = filtered.and(
                    apply_restriction(card, group1, cards, cache)
                        .or(apply_restriction(card, group2, cards, cache)),
                );
            }
            QueryRestriction::Group(group) => {
                filtered = filtered.and(apply_restriction(card, group, cards, cache));
            }
            QueryRestriction::Fuzzy(x) => {
                filtered = filtered.and(if search::fuzzy(card, x) {
                    QueryMatch::Match
                } else {
                    QueryMatch::NotMatch
                });
            }
            QueryRestriction::Comparison(field, comparison) => {
                filtered = filtered.and(comparison.maybe_compare(card.get_num_property(field)));
            }
            QueryRestriction::Contains(field, contains) => {
                let matches = match card.get_str_property(field) {
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
                filtered = filtered.and(!apply_restriction(card, queryres, cards, cache));
            }
            QueryRestriction::LenientNot(queryres) => {
                filtered = filtered.and(
                    if apply_restriction(card, queryres, cards, cache) == QueryMatch::Match {
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
                            apply_restriction(&devoured_id, query, cards, cache)
                                == QueryMatch::Match
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
                            apply_restriction(card, devoured_by, &cloned_cards, cache)
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

                    devoured_cards = apply_restrictions(&devourees_query, cloned_cards);
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

/// This function is used to check whether any part of an optional `vec` fulfills a `cond`ition.
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

/// Function that takes `cards` and outputs a vector pointing to all the cards that matched the queries.
#[must_use]
pub fn apply_restrictions<'a, 'b, C, I>(query: &Query, cards: I) -> Vec<&'a C>
where
    C: ReadProperties + Clone + 'a,
    I: IntoIterator<Item = &'a C> + Clone + 'b,
    &'a C: ReadProperties,
{
    let cards_clone = cards.clone();
    let cache = Cache::new(HashMap::new());
    let mut results: Vec<&C> = cards
        .into_iter()
        .filter(|card| apply_restriction(card, query, &cards_clone, &cache) == QueryMatch::Match)
        .collect();

    match &query.sort {
        Sort::None => (),
        Sort::Fuzzy if !query.name.is_empty() => results.sort_by(|a, b| {
            weighted_compare(b, &query.name)
                .partial_cmp(&weighted_compare(a, &query.name))
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        Sort::Fuzzy => results.sort_by(|a, b| Ord::cmp(&a.get_name(), &b.get_name())),
        Sort::Alphabet(property, search::Ordering::Ascending) => results
            .sort_by(|a, b| Ord::cmp(&a.get_str_property(property), &b.get_str_property(property))),
        Sort::Numeric(property, search::Ordering::Ascending) => results
            .sort_by(|a, b| Ord::cmp(&a.get_num_property(property), &b.get_num_property(property))),
        Sort::Alphabet(property, search::Ordering::Descending) => {
            results.sort_by(|a, b| {
                Ord::cmp(&a.get_str_property(property), &b.get_str_property(property)).reverse()
            });
        }
        Sort::Numeric(property, search::Ordering::Descending) => results.sort_by(|a, b| {
            Ord::cmp(&a.get_num_property(property), &b.get_num_property(property)).reverse()
        }),
    }

    results
}

/// Compares a card's text with a given string and outputs a value for how much it matched the text, prioritizing in this order: Names, types, descriptions, kins, keywords.
/// Notably, since a card's keywords are also in its description, keywords are ranked slightly higher than they are supposed to. This is not a huge deal, but it is a thing that might be good to be aware of.
#[must_use]
pub fn weighted_compare(a: &impl ReadProperties, b: &str) -> f32 {
    let mut result = 0.0;

    if let Some(name) = a.get_name() {
        result += fuzzy_compare(name, b) * 3.;
    }

    if let Some(r#type) = a.get_type() {
        result += fuzzy_compare(r#type, b) * 1.8;
    }

    if let Some(description) = a.get_description() {
        result += fuzzy_compare(description, b) * 1.6;
    }

    if let Some(kins) = a.get_kins() {
        result += kins
            .iter()
            .map(|x| fuzzy_compare(x, b))
            .max_by(|a, b| PartialOrd::partial_cmp(a, b).unwrap_or(Ordering::Less))
            .unwrap_or(0.0)
            * 1.5;
    }

    if let Some(keywords) = a.get_keywords() {
        result += keywords
            .iter()
            .map(|x| fuzzy_compare(&x.name, b))
            .max_by(|a, b| PartialOrd::partial_cmp(a, b).unwrap_or(Ordering::Less))
            .unwrap_or(0.0);
    }

    result
}

#[cfg(test)]
mod test {
    use std::{fmt::Display, fs};

    use crate::{apply_restrictions, cards::Card, search::query_parser};

    struct PrintableCards<'a>(Vec<&'a Card>);

    impl<'a> Display for PrintableCards<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            for card in &self.0 {
                writeln!(f, "{card}")?;
                writeln!(f)?;
            }
            Ok(())
        }
    }

    #[test]
    fn test_search() {
        let data =
            fs::read_to_string("../hemolymph-server/cards.json").expect("Unable to read file");
        let cards: Vec<Card> = serde_json::from_str(&data).expect("Unable to parse JSON");
        let parsed = query_parser::query_parser("n:infected OR dby:(n:\"infected fly\")").unwrap();
        println!("{parsed}");
        // println!("{parsed:#?}");
        let cards = PrintableCards(apply_restrictions(&parsed, cards.iter()));
        println!("{cards}");
    }
}
