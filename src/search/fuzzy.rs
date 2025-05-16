use std::cmp::{max_by, Ordering};

use rust_fuzzy_search::fuzzy_compare;

use crate::cards::properties::Read;

/// Compares a card's text with a given string and outputs a value for how much it matched the text.
///
/// The comparison prioritizes in this order: Names, types, descriptions, kins, keywords.
///
/// Notably, since a card's keywords are also in its description, keywords are ranked slightly higher than they are supposed to. This is not a huge deal, but it is a thing that might be good to be aware of.
#[must_use]
pub fn weighted_compare(a: &impl Read, b: &str) -> f32 {
    let mut result = 0.0;

    if let Some(name) = a.get_name() {
        result += max_by(
            fuzzy_compare(name, b),
            fuzzy_compare(&name.to_lowercase(), &b.to_lowercase()),
            f32::total_cmp,
        ) * 3.;
    }

    if let Some(r#type) = a.get_type() {
        result += fuzzy_compare(&r#type.to_lowercase(), &b.to_lowercase()) * 1.8;
    }

    if let Some(description) = a.get_description() {
        result += fuzzy_compare(&description.to_string().to_lowercase(), &b.to_lowercase()) * 1.6;
    }

    if let Some(kin) = a.get_kin() {
        result += fuzzy_compare(kin.get_name(), b) * 1.5;
    }

    if let Some(keywords) = a.get_keywords() {
        result += keywords
            .iter()
            .map(|x| fuzzy_compare(&x.name.to_lowercase(), &b.to_lowercase()))
            .max_by(|a, b| PartialOrd::partial_cmp(a, b).unwrap_or(Ordering::Less))
            .unwrap_or(0.0);
    }

    result
}
