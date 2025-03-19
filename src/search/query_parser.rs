use chumsky::{
    error::Simple,
    prelude::{choice, end, filter, just, recursive},
    text::{ident, keyword, TextParser},
    Parser,
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
pub fn parse_query(string: &str) -> Result<Query, Vec<Simple<char>>> {
    let parser = make_query_parser();
    parser.parse(string)
}

#[allow(clippy::too_many_lines)]
fn make_query_parser() -> impl Parser<char, Query, Error = Simple<char>> {
    let name_property = choice((keyword("name"), keyword("n"))).map(|()| Text::Name);
    let id_property = keyword("id").map(|()| Text::Id);
    let type_property = choice((keyword("type"), keyword("t"))).map(|()| Text::Id);
    let description_property = choice((keyword("description"), keyword("desc"), keyword("dc")))
        .map(|()| Text::Description);
    let flavor_text_property =
        choice((keyword("flavor_text"), keyword("ft"))).map(|()| Text::FlavorText);

    let s_text_property = choice((
        name_property,
        id_property,
        type_property,
        description_property,
        flavor_text_property,
    ))
    .padded();

    let ascending =
        choice((keyword("ASCENDING"), keyword("ASC"), keyword("A"))).map(|()| Ordering::Ascending);
    let descending =
        choice((keyword("DESCENDING"), keyword("DES"), keyword("D"))).map(|()| Ordering::Ascending);
    let sort_dir = choice((
        ascending.map(|_| Ordering::Ascending),
        descending.map(|_| Ordering::Ascending),
    ))
    .padded();

    let cost_property = choice((keyword("c"), keyword("cost"))).map(|()| Number::Cost);
    let strength_property = choice((
        keyword("s"),
        keyword("p"),
        keyword("strenght"),
        keyword("power"),
    ))
    .map(|()| Number::Power);
    let defense_property =
        choice((keyword("d"), keyword("defense"), keyword("def"))).map(|()| Number::Defense);
    let health_property =
        choice((keyword("h"), keyword("hp"), keyword("health"))).map(|()| Number::Health);

    let s_num_property = choice((
        cost_property,
        strength_property,
        defense_property,
        health_property,
    ))
    .padded();

    let sortable_property = choice((
        s_num_property.clone().map(Properties::NumProperty),
        s_text_property.clone().map(Properties::StringProperty),
    ));

    let sort_expr = keyword("SORT")
        .padded()
        .ignore_then(sort_dir)
        .then_ignore(keyword("BY").padded())
        .then(sortable_property)
        .map(|(ordering, property)| match property {
            Properties::StringProperty(property) => Sort::Alphabet(property, ordering),
            Properties::NumProperty(property) => Sort::Numeric(property, ordering),
            _ => unreachable!("The parser is incapable of representing this state."),
        });

    let num_property = s_num_property.clone();
    let text_property = s_text_property.clone();

    recursive(|query_rec| {
        let devours_property = choice((keyword("devours"), keyword("dev"), keyword("dv"))).padded();
        let devours_compare = devours_property
            .ignore_then(just(':'))
            .ignore_then(query_rec.clone().delimited_by(just('('), just(')')))
            .map(QueryRestriction::Devours);

        let devoured_by_property = choice((keyword("devoured_by"), keyword("dby"))).padded();
        let devoured_by_compare = devoured_by_property
            .ignore_then(just(':'))
            .ignore_then(query_rec.clone().delimited_by(just('('), just(')')))
            .map(QueryRestriction::DevouredBy);

        let quoted_text = filter(|a| char::is_alphanumeric(*a))
            .repeated()
            .collect()
            .delimited_by(just('"'), just('"'));

        let regex = filter(|a| char::is_alphanumeric(*a))
            .repeated()
            .collect()
            .delimited_by(just('/'), just('/'))
            .map(|x: String| Regex::new(&x));

        let text_comparable = choice((
            ident().map(TextComparable::String),
            quoted_text.map(TextComparable::String),
            regex
                .try_map(|x, span| x.map_err(|err| Simple::custom(span, format!("{err}"))))
                .map(TextComparable::Regex),
        ));

        let text_comparison_symbol = choice((
            just('=').map(|_| TextComparisonSymbol::Equals),
            just(':').map(|_| TextComparisonSymbol::Contains),
        ))
        .padded();

        let text_comparison =
            text_comparison_symbol
                .padded()
                .then(text_comparable)
                .map(|(comparison, compared)| match compared {
                    TextComparable::String(text) => match comparison {
                        TextComparisonSymbol::Contains => TextComparison::Contains(text),
                        TextComparisonSymbol::Equals => TextComparison::EqualTo(text),
                    },
                    TextComparable::Regex(regex) => TextComparison::HasMatch(regex),
                });

        let text_property_comparison = text_property
            .then(text_comparison)
            .map(|(property, comparison)| QueryRestriction::TextComparison(property, comparison));

        let nat_number = filter(|x: &char| x.is_numeric())
            .repeated()
            .at_least(1)
            .collect()
            .try_map(|text: String, span| {
                text.parse::<usize>()
                    .map_err(|err| Simple::custom(span, format!("{err}")))
            })
            .padded();

        let number_comparison_symbol = choice((
            just(">=").map(|_| NumberComparisonSymbol::GreaterThanOrEqual),
            just("<=").map(|_| NumberComparisonSymbol::LessThanOrEqual),
            just('>').map(|_| NumberComparisonSymbol::GreaterThan),
            just('<').map(|_| NumberComparisonSymbol::LessThan),
            just("!=").map(|_| NumberComparisonSymbol::NotEqual),
            just("=").map(|_| NumberComparisonSymbol::Equal),
            just(":").map(|_| NumberComparisonSymbol::Equal),
        ))
        .padded();

        let number_comparison = number_comparison_symbol
            .then(nat_number)
            .map(|(comp, num)| match comp {
                NumberComparisonSymbol::GreaterThan => Comparison::GreaterThan(num),
                NumberComparisonSymbol::LessThan => Comparison::LowerThan(num),
                NumberComparisonSymbol::GreaterThanOrEqual => Comparison::GreaterThanOrEqual(num),
                NumberComparisonSymbol::LessThanOrEqual => Comparison::LowerThanOrEqual(num),
                NumberComparisonSymbol::Equal => Comparison::Equal(num),
                NumberComparisonSymbol::NotEqual => Comparison::NotEqual(num),
            });

        let number_property_comparison = num_property
            .then(number_comparison)
            .map(|(property, comparison)| QueryRestriction::NumberComparison(property, comparison));

        let fuzzy_text = choice((
            filter(|x: &char| x.is_ascii_alphanumeric())
                .repeated()
                .at_least(1)
                .collect(),
            quoted_text,
        ))
        .map(QueryRestriction::Fuzzy);

        let or_expressions = query_rec
            .clone()
            .then_ignore(keyword("OR"))
            .then(query_rec.clone())
            .delimited_by(just('(').padded(), just(')').padded())
            .map(|(left, right)| QueryRestriction::Or(left, right));

        let xor_expressions = query_rec
            .clone()
            .then_ignore(keyword("XOR"))
            .then(query_rec.clone())
            .delimited_by(just('(').padded(), just(')').padded())
            .map(|(left, right)| QueryRestriction::Xor(left, right));

        let not_expressions = just('-')
            .padded()
            .ignore_then(query_rec.clone())
            .map(QueryRestriction::Not);

        let super_not_expressions = just('!')
            .padded()
            .ignore_then(query_rec.clone())
            .map(QueryRestriction::LenientNot);

        let kins = choice((keyword("kin"), keyword("kins"), keyword("k"))).map(|()| Array::Kins);
        let functions = choice((
            keyword("functions"),
            keyword("function"),
            keyword("funs"),
            keyword("fns"),
            keyword("fn"),
            keyword("fun"),
        ))
        .map(|()| Array::Functions);

        let array_property = choice((kins, functions));

        let array_property_comparison = array_property
            .then_ignore(text_comparison_symbol)
            .then(ident().or(quoted_text))
            .map(|(property, value)| QueryRestriction::Has(property, value));

        let keywords = choice((keyword("kw"), keyword("keyword")));

        let keyword_property_comparison = keywords
            .ignore_then(text_comparison_symbol.ignore_then(ident().or(quoted_text)))
            .map(QueryRestriction::HasKw);

        let query = choice((
            not_expressions,
            super_not_expressions,
            or_expressions,
            xor_expressions,
            text_property_comparison,
            number_property_comparison,
            array_property_comparison,
            keyword_property_comparison,
            devoured_by_compare,
            devours_compare,
            fuzzy_text,
        ))
        .padded()
        .repeated();

        choice((
            query.clone().at_least(1),
            query.delimited_by(just('(').padded(), just(')').padded()),
        ))
        .repeated()
        .flatten()
        .collect::<Vec<QueryRestriction>>()
        .map(query_from_restrictions)
    })
    .then(sort_expr.or_not())
    .map(|(mut query, sorting)| {
        if let Some(sorting) = sorting {
            query.sort = sorting;
        }
        query
    })
    .then_ignore(end())
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

enum TextComparisonSymbol {
    Contains,
    Equals,
}

enum NumberComparisonSymbol {
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
    NotEqual,
}
