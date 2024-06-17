#![warn(clippy::pedantic)]

use std::cmp::Ordering;

use cards::{KeywordData, ReadProperties};
use rust_fuzzy_search::fuzzy_compare;
use search::{Query, QueryRestriction, Sort};

pub mod cards;
pub mod search;

fn apply_restriction(card: &impl ReadProperties, query: &Query) -> bool {
    let mut filtered = true;
    for res in &query.restrictions {
        match res {
            QueryRestriction::Group(group) => {
                filtered = filtered && apply_restriction(card, group);
            }
            QueryRestriction::Fuzzy(x) => {
                filtered = filtered && search::fuzzy(card, x);
            }
            QueryRestriction::Comparison(field, comparison) => {
                filtered =
                    filtered && comparison.maybe_compare(card.get_num_property(field), false);
            }
            QueryRestriction::Contains(field, contains) => {
                filtered = filtered
                    && card.get_str_property(field).is_some_and(|x| {
                        x.to_lowercase().contains(contains.to_lowercase().as_str())
                    });
            }
            QueryRestriction::Has(field, thing) => {
                let x = card.get_vec_property(field);
                filtered = filtered && x.is_some_and(|x| x.iter().any(|x| x.contains(thing)));
            }
            QueryRestriction::HasKw(thing) => {
                let x = card.get_keywords();
                filtered = filtered && x.is_some_and(|x| x.iter().any(|x| x.name.contains(thing)));
            }
            QueryRestriction::Not(queryres) => {
                filtered = filtered && !apply_restriction(card, queryres);
            }
            QueryRestriction::Devours(query) => {
                filtered = filtered
                    && card.get_keywords().is_some_and(|x| {
                        x.iter().any(|x| {
                            if x.name == "devours" {
                                if let Some(KeywordData::CardID(ref devoured_id)) = x.data {
                                    apply_restriction(&devoured_id, query)
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        })
                    });
            }
        }
    }
    filtered
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
        .filter(|card| apply_restriction(card, query))
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
    use crate::{apply_restrictions, cards::Card, search::query_parser};

    #[test]
    fn test_search() {
        let cards = vec![
            Card {
                id: "1".to_string(),
                name: "bap".to_string(),
                img: vec![],
                description: "thoinasdf".to_string(),
                cost: 5,
                health: 1,
                defense: 2,
                power: 4,
                r#type: "awa".to_string(),
                ..Default::default()
            },
            Card {
                id: "2".to_string(),
                name: "bap".to_string(),
                img: vec![],
                description: "was".to_string(),
                cost: 5,
                health: 1,
                defense: 2,
                power: 4,
                r#type: "awa".to_string(),
                ..Default::default()
            },
            Card {
                id: "4".to_string(),
                name: "bap".to_string(),
                img: vec![],
                description: "thoinasdf".to_string(),
                cost: 3,
                health: 1,
                defense: 2,
                power: 4,
                r#type: "awa".to_string(),
                ..Default::default()
            },
            Card {
                id: "4".to_string(),
                name: "bop".to_string(),
                img: vec![],
                description: "thoinasdf".to_string(),
                cost: 3,
                health: 1,
                defense: 2,
                power: 4,
                r#type: "awa".to_string(),
                ..Default::default()
            },
        ];
        let parsed = query_parser::query_parser("-(n:bap c=5)");
        println!("{parsed:#?}");
        let cards = parsed.map(|res| apply_restrictions(&res, cards.iter()));
        println!("{cards:#?}");
    }
}
