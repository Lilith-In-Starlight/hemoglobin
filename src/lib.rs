#![warn(clippy::pedantic)]
use rust_fuzzy_search::fuzzy_compare;

use cards::Card;
use search::QueryRestriction;

pub mod cards;
pub mod search;

#[must_use]
pub fn apply_restrictions<'a>(
    query_restrictions: &[QueryRestriction],
    cards: &'a [Card],
) -> Vec<&'a Card> {
    let mut name = String::new();
    let mut results: Vec<&Card> = cards
        .iter()
        .filter(|card| {
            let mut filtered = true;
            for res in query_restrictions {
                match res {
                    QueryRestriction::Fuzzy(x) => {
                        filtered = filtered && search::fuzzy(card, x);
                        name.clone_from(x);
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
                }
            }
            filtered
        })
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
