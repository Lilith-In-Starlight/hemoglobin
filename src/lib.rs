#![warn(clippy::pedantic)]

use std::{
    cmp::{max, min, Ordering},
    ops::Not,
};

use cards::{KeywordData, ReadProperties};
use rust_fuzzy_search::fuzzy_compare;
use search::{Query, QueryRestriction, Sort};

pub mod cards;
pub mod search;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum QueryMatch {
    NotHave,
    NotMatch,
    Match,
}

impl QueryMatch {
    fn or(self, b: Self) -> Self {
        max(self, b)
    }
    fn xor(self, b: Self) -> Self {
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
    fn and(self, b: Self) -> Self {
        min(self, b)
    }
}

impl Not for QueryMatch {
    type Output = QueryMatch;

    fn not(self) -> Self::Output {
        match self {
            Self::Match => Self::NotMatch,
            Self::NotMatch => Self::Match,
            Self::NotHave => Self::NotHave,
        }
    }
}

fn apply_restriction(card: &impl ReadProperties, query: &Query) -> QueryMatch {
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
                let res1 = apply_restriction(card, group1);
                let res2 = apply_restriction(card, group2);
                filtered = filtered.and(res1.xor(res2));
            }
            QueryRestriction::Or(group1, group2) => {
                filtered = filtered
                    .and(apply_restriction(card, group1).or(apply_restriction(card, group2)));
            }
            QueryRestriction::Group(group) => {
                filtered = filtered.and(apply_restriction(card, group));
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
                filtered = filtered.and(!apply_restriction(card, queryres));
            }
            QueryRestriction::LenientNot(queryres) => {
                filtered =
                    filtered.and(if apply_restriction(card, queryres) == QueryMatch::Match {
                        QueryMatch::NotMatch
                    } else {
                        QueryMatch::Match
                    });
            }
            QueryRestriction::Devours(query) => {
                let matches = match_in_vec(card.get_keywords(), |keyword| {
                    if keyword.name == "devours" {
                        if let Some(KeywordData::CardID(ref devoured_id)) = keyword.data {
                            apply_restriction(&devoured_id, query) == QueryMatch::Match
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                });
                filtered = filtered.and(matches);
            }
        }
    }
    filtered
}

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

#[must_use]
pub fn apply_restrictions<'a, C: ReadProperties + 'a, T: Iterator<Item = &'a C>>(
    query: &Query,
    cards: T,
) -> Vec<&'a C>
where
    &'a C: ReadProperties,
{
    let mut results: Vec<&C> = cards
        .filter(|card| apply_restriction(card, query) == QueryMatch::Match)
        .collect();

    match &query.sort {
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
        Sort::Alphabet(property, search::Ordering::Descending) => results.sort_by(|a, b| {
            Ord::cmp(&a.get_str_property(property), &b.get_str_property(property)).reverse()
        }),
        Sort::Numeric(property, search::Ordering::Descending) => results.sort_by(|a, b| {
            Ord::cmp(&a.get_num_property(property), &b.get_num_property(property)).reverse()
        }),
    }

    results
}

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
        let parsed = query_parser::query_parser("n:/.* ant/");
        println!("{parsed:#?}");
        let cards = PrintableCards(
            parsed
                .map(|res| apply_restrictions(&res, cards.iter()))
                .unwrap(),
        );
        println!("{cards}");
    }
}
