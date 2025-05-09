use chumsky::{
    error::Rich,
    extra,
    prelude::{any, choice, just, recursive},
    text::ascii::keyword,
    IterParser, Parser,
};
use regex::Regex;

use crate::{
    cards::properties::{Array, Number, Text},
    numbers::Comparison,
};

use super::{Errors, Ordering, Query, QueryRestriction, Sort, TextComparison};

/// # Errors
/// When `str` is not a valid property query name
pub fn get_property_from_name(str: &str) -> Result<Properties, Errors> {
    match str {
        "id" => Ok(Properties::StringProperty(Text::Id)),
        "name" | "n" => Ok(Properties::StringProperty(Text::Name)),
        "flavortext" | "flavor" | "ft" => Ok(Properties::StringProperty(Text::FlavorText)),
        "description" | "desc" | "de" => Ok(Properties::StringProperty(Text::Description)),
        "type" | "t" => Ok(Properties::StringProperty(Text::Type)),
        "cost" | "c" => Ok(Properties::NumProperty(Number::Cost)),
        "health" | "h" | "hp" => Ok(Properties::NumProperty(Number::Health)),
        "power" | "strength" | "damage" | "p" | "dmg" | "str" => {
            Ok(Properties::NumProperty(Number::Power))
        }
        "defense" | "defence" | "def" | "d" => Ok(Properties::NumProperty(Number::Defense)),
        "kin" | "k" => Ok(Properties::ArrayProperty(Array::Kins)),
        "function" | "fun" | "fn" | "f" => Ok(Properties::ArrayProperty(Array::Functions)),
        "keyword" | "kw" => Ok(Properties::Keywords),
        "sort" | "so" => Ok(Properties::Sort(Ordering::Ascending)),
        "sortd" | "sod" => Ok(Properties::Sort(Ordering::Descending)),
        _ => Err(Errors::UnknownStringParam(str.to_owned())),
    }
}

pub enum Properties {
    NumProperty(Number),
    StringProperty(Text),
    ArrayProperty(Array),
    Sort(Ordering),
    Keywords,
}

pub(crate) fn text_comparison_parser(s: &str) -> Result<Comparison, Errors> {
    s.parse::<usize>().map_or_else(
        |_| {
            #[allow(clippy::option_if_let_else)]
            if let Some(end) = s.strip_prefix(">=") {
                end.parse::<usize>()
                    .map(Comparison::GreaterThanOrEqual)
                    .map_err(|_| Errors::InvalidComparisonString)
            } else if let Some(end) = s.strip_prefix("<=") {
                end.parse::<usize>()
                    .map(Comparison::LowerThanOrEqual)
                    .map_err(|_| Errors::InvalidComparisonString)
            } else if let Some(end) = s.strip_prefix('>') {
                end.parse::<usize>()
                    .map(Comparison::GreaterThan)
                    .map_err(|_| Errors::InvalidComparisonString)
            } else if let Some(end) = s.strip_prefix('<') {
                end.parse::<usize>()
                    .map(Comparison::LowerThan)
                    .map_err(|_| Errors::InvalidComparisonString)
            } else if let Some(end) = s.strip_prefix('=') {
                end.parse::<usize>()
                    .map(Comparison::Equal)
                    .map_err(|_| Errors::InvalidComparisonString)
            } else if let Some(end) = s.strip_prefix("!=") {
                end.parse::<usize>()
                    .map(Comparison::NotEqual)
                    .map_err(|_| Errors::InvalidComparisonString)
            } else {
                Err(Errors::InvalidComparisonString)
            }
        },
        |x| Ok(Comparison::Equal(x)),
    )
}

enum TextComparable {
    String(String),
    Regex(Regex),
}

/// Parses a query.
/// # Errors
/// If the parser fails.
pub fn parse_query<'a>(string: &'a str) -> Result<Query, Vec<Rich<'a, char>>> {
    let parser = make_query_parser();
    parser.parse(string).into_result()
}

#[allow(clippy::too_many_lines)]
fn make_query_parser<'a>() -> impl Parser<'a, &'a str, Query, extra::Err<Rich<'a, char>>> {
    let word = any()
        .filter(|c: &char| c.is_ascii_alphanumeric())
        .repeated()
        .at_least(1)
        .collect::<String>();

    let number = any()
        .filter(|c: &char| c.is_numeric())
        .repeated()
        .at_least(1)
        .collect::<String>()
        .try_map(|x, span| x.parse().map_err(|x| Rich::custom(span, "Not a number")));

    let regex_text = word
        .padded()
        .repeated()
        .collect::<Vec<String>>()
        .try_map(|x, span| {
            let str = x
                .into_iter()
                .reduce(|mut acc, el| {
                    acc += &el;
                    acc
                })
                .unwrap_or_default();

            Regex::new(str.as_str()).map_err(|x| Rich::custom(span, "Not a valid regex"))
        })
        .delimited_by(just('/'), just('/'));

    let expr = recursive(|expr| {
        let quoted_text = word
            .padded()
            .repeated()
            .collect::<Vec<String>>()
            .map(|x| {
                x.into_iter()
                    .reduce(|mut acc, el| {
                        acc += &el;
                        acc
                    })
                    .unwrap_or_default()
            })
            .delimited_by(just('"'), just('"'));

        // Num Properties
        let cost_property_name = keyword("cost").to(Number::Cost);
        let power_property_name = keyword("power").to(Number::Power);
        let def_property_name = keyword("def").to(Number::Defense);
        let health_property_name = keyword("health").to(Number::Health);

        let num_property_name = choice((
            cost_property_name,
            power_property_name,
            def_property_name,
            health_property_name,
        ));

        let num_comparison_symbol = choice((
            just('>').to(NumberComparisonSymbol::GreaterThan),
            just('<').to(NumberComparisonSymbol::LessThan),
            just('=').to(NumberComparisonSymbol::Equal),
            just('!')
                .then(just('='))
                .to(NumberComparisonSymbol::NotEqual),
        ));

        let num_comparison = num_comparison_symbol
            .then(number)
            .map(|(comparison, number)| match comparison {
                NumberComparisonSymbol::GreaterThan => Comparison::GreaterThan(number),
                NumberComparisonSymbol::LessThan => Comparison::LowerThan(number),
                NumberComparisonSymbol::GreaterThanOrEqual => {
                    Comparison::GreaterThanOrEqual(number)
                }
                NumberComparisonSymbol::LessThanOrEqual => Comparison::LowerThanOrEqual(number),
                NumberComparisonSymbol::Equal => Comparison::Equal(number),
                NumberComparisonSymbol::NotEqual => Comparison::NotEqual(number),
            });

        let num_property = num_property_name
            .then(num_comparison)
            .map(|(property, cost)| QueryRestriction::NumberComparison(property, cost));

        // Text Properties

        let name_property_name = keyword("name").to(Text::Name);
        let desc_property_name = keyword("desc").to(Text::Description);
        let flavor_property_name = keyword("flavor").to(Text::FlavorText);
        let id_property_name = keyword("id").to(Text::Id);
        let type_property_name = keyword("type").to(Text::Type);

        let text_property_name = choice((
            name_property_name,
            desc_property_name,
            flavor_property_name,
            id_property_name,
            type_property_name,
        ));

        let text_comparison_symbol = choice((
            just('=').to(TextComparisonSymbol::Equals),
            just(':').to(TextComparisonSymbol::Contains),
        ));

        let text_comparable = choice((
            quoted_text.map(TextComparable::String),
            regex_text.map(TextComparable::Regex),
        ));

        let text_comparison = text_comparison_symbol
            .then(text_comparable)
            .map(|(symbol, text)| match text {
                TextComparable::String(string) => match symbol {
                    TextComparisonSymbol::Contains => TextComparison::Contains(string),
                    TextComparisonSymbol::Equals => TextComparison::EqualTo(string),
                },
                TextComparable::Regex(regex) => TextComparison::HasMatch(regex),
            });

        let text_property = text_property_name
            .then(text_comparison)
            .map(|(property, comparison)| QueryRestriction::TextComparison(property, comparison));

        let atom = choice((num_property, text_property)).map(|x| query_from_restrictions(vec![x]));

        let atom = atom.or(expr.clone().delimited_by(just('('), just(')')));

        atom.then(just("OR").ignore_then(expr).or_not()).map(
            |(first, maybe_second): (Query, Option<Query>)| match maybe_second {
                None => first,
                Some(right) => query_from_restrictions(vec![QueryRestriction::Or(first, right)]),
            },
        )
    });

    expr
}

fn query_from_restrictions(restrictions: Vec<QueryRestriction>) -> Query {
    let mut name = String::new();

    for restriction in &restrictions {
        if let QueryRestriction::Fuzzy(a) = restriction {
            name += a;
            name += " ";
        }
    }

    let sort = if name.is_empty() {
        Sort::Alphabet(Text::Name, Ordering::Ascending)
    } else {
        Sort::Fuzzy
    };

    Query {
        name,
        restrictions,
        sort,
    }
}

#[derive(Clone)]
enum TextComparisonSymbol {
    Contains,
    Equals,
}

#[derive(Clone)]
enum NumberComparisonSymbol {
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

enum QueryOp {
    Not,
    LenientNot,
}
enum QueryBinOp {
    Or,
    Xor,
}
