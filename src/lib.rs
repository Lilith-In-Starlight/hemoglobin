#![warn(clippy::pedantic)]
use rust_fuzzy_search::fuzzy_compare;

use cards::Card;
use search::QueryRestriction;

pub mod cards;
pub mod search;

fn apply_restriction(card: &Card, query_restrictions: &[QueryRestriction]) -> bool {
    let mut filtered = true;
    for res in query_restrictions {
        match res {
            QueryRestriction::Fuzzy(x) => {
                filtered = filtered && search::fuzzy(card, x);
            }
            QueryRestriction::Comparison(field, comparison) => {
                filtered = filtered && comparison.compare(&field(card));
            }
            QueryRestriction::Contains(what, contains) => {
                filtered = filtered
                    && what(card)
                        .to_lowercase()
                        .contains(contains.to_lowercase().as_str());
            }
            QueryRestriction::Has(fun, thing) => {
                let x = fun(card);
                filtered = filtered && x.iter().any(|x| x.contains(thing));
            }
            QueryRestriction::HasKw(fun, thing) => {
                let x = fun(card);
                filtered = filtered && x.iter().any(|x| x.name.contains(thing));
            }
            QueryRestriction::Not(queryres) => {
                filtered = filtered && !apply_restriction(card, queryres);
            }
        }
    }
    filtered
}

#[must_use]
pub fn apply_restrictions<'a, T: Iterator<Item = &'a Card>>(
    query_restrictions: &[QueryRestriction],
    cards: T,
) -> Vec<&'a Card> {
    let name = query_restrictions
        .iter()
        .filter_map(|x| {
            if let QueryRestriction::Fuzzy(x) = x {
                Some(x)
            } else {
                None
            }
        })
        .last()
        .cloned()
        .unwrap_or_default();

    let mut results: Vec<&Card> = cards
        .filter(|card| !apply_restriction(card, query_restrictions))
        .collect();

    results.sort_by(|a, b| {
        if name.is_empty() {
            Ord::cmp(&a.name, &b.name)
        } else {
            weighted_compare(b, &name)
                .partial_cmp(&weighted_compare(a, &name))
                .unwrap_or(std::cmp::Ordering::Equal)
        }
    });

    results
}

#[must_use]
pub fn weighted_compare(a: &Card, b: &str) -> f32 {
    fuzzy_compare(&a.name, b) * 3.
        + fuzzy_compare(&a.r#type, b) * 1.8
        + fuzzy_compare(&a.description, b) * 1.6
        + a.kins
            .iter()
            .map(|x| fuzzy_compare(x, b))
            .max_by(|a, b| PartialOrd::partial_cmp(a, b).unwrap_or(std::cmp::Ordering::Less))
            .unwrap_or(0.0)
            * 1.5
        + a.keywords
            .iter()
            .map(|x| fuzzy_compare(&x.name, b))
            .max_by(|a, b| PartialOrd::partial_cmp(a, b).unwrap_or(std::cmp::Ordering::Less))
            .unwrap_or(0.0)
            * 1.2
}

#[cfg(test)]
mod test {
    use crate::{apply_restrictions, cards::Card, search::query_parser};

    #[test]
    fn test_search() {
        let cards = vec![
            Card {
                id: "1".to_string(),
                name: "thing".to_string(),
                img: vec![],
                description: "thoinasdf".to_string(),
                cost: 1,
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
                description: "thoinasdf".to_string(),
                cost: 5,
                health: 1,
                defense: 2,
                power: 4,
                r#type: "awa".to_string(),
                ..Default::default()
            },
        ];
        let cards =
            query_parser::query_parser("bap").map(|res| apply_restrictions(&res, cards.iter()));
        println!("{cards:#?}");
    }
}
