use chumsky::{
    error::Rich,
    extra,
    label::LabelError,
    prelude::{any, choice, end, just, recursive},
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
pub fn parse_query(string: &str) -> Result<Query, Vec<Rich<'_, char>>> {
    let parser = make_query_parser();
    parser.parse(string).into_result()
}

#[allow(clippy::too_many_lines)]
fn make_query_parser<'a>() -> impl Parser<'a, &'a str, Query, extra::Err<Rich<'a, char>>> + 'a {
    let word = any()
        .filter(|c: &char| c.is_ascii_alphanumeric())
        .labelled("alphanumeric")
        .repeated()
        .at_least(1)
        .collect::<String>()
        .labelled("ident");

    let keyword = |mat: &'static str| {
        word.try_map(move |kw, span| {
            if mat == kw {
                Ok(())
            } else {
                let a = LabelError::<'a, &'a str, String>::expected_found(
                    [mat.to_string()],
                    None,
                    span,
                );
                Err(a)
            }
        })
        .labelled(format!("`{mat}`"))
    };

    let name_property_name = choice((keyword("name"), keyword("n"))).to(Text::Name);
    let desc_property_name =
        choice((keyword("description"), keyword("desc"), keyword("d"))).to(Text::Description);
    let flavor_property_name =
        choice((keyword("flavortext"), keyword("flavor"), keyword("ft"))).to(Text::FlavorText);
    let id_property_name = keyword("id").to(Text::Id);
    let type_property_name = choice((keyword("type"), keyword("t"))).to(Text::Type);

    let text_property_name = choice((
        name_property_name,
        desc_property_name,
        flavor_property_name,
        id_property_name,
        type_property_name,
    ))
    .padded();

    let cost_property_name = choice((keyword("cost"), keyword("c"))).to(Number::Cost);
    let power_property_name = choice((keyword("power"), keyword("p"))).to(Number::Power);
    let def_property_name =
        choice((keyword("defense"), keyword("def"), keyword("d"))).to(Number::Defense);
    let health_property_name =
        choice((keyword("health"), keyword("hp"), keyword("h"))).to(Number::Health);

    let num_property_name = choice((
        cost_property_name,
        power_property_name,
        def_property_name,
        health_property_name,
    ));

    let number = any()
        .filter(|c: &char| c.is_numeric())
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map_err(|err: Rich<_>| {
            LabelError::<'a, &'a str, String>::expected_found(
                ["number".to_string()],
                None,
                *err.span(),
            )
        })
        .try_map(|x, span| {
            x.parse()
                .map_err(|x| Rich::custom(span, format!("Not a number: {x}")))
        })
        .labelled("number");

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

            Regex::new(str.as_str())
                .map_err(|x| Rich::custom(span, format!("Not a valid regex: {x}")))
        })
        .delimited_by(just('/'), just('/'));

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
        .delimited_by(just('"'), just('"'))
        .labelled("quoted text");

    let expr = recursive(|expr| {
        let group = expr
            .clone()
            .repeated()
            .collect()
            .delimited_by(just('(').padded(), just(')').padded())
            .labelled("subquery");

        let group_restriction = group
            .clone()
            .map(|x| QueryRestriction::Group(query_from_restrictions(x)));

        let group_query = group.clone().map(query_from_restrictions);

        // Num Properties
        let num_comparison_symbol = choice((
            just("!=").to(NumberComparisonSymbol::NotEqual),
            just(">=").to(NumberComparisonSymbol::GreaterThanOrEqual),
            just("<=").to(NumberComparisonSymbol::LessThanOrEqual),
            just('>').to(NumberComparisonSymbol::GreaterThan),
            just('<').to(NumberComparisonSymbol::LessThan),
            just('=').to(NumberComparisonSymbol::Equal),
        ))
        .labelled("comparison operator");

        let num_comparison = num_comparison_symbol
            .padded()
            .then(number.padded())
            .map(|(comparison, number)| match comparison {
                NumberComparisonSymbol::GreaterThan => Comparison::GreaterThan(number),
                NumberComparisonSymbol::LessThan => Comparison::LowerThan(number),
                NumberComparisonSymbol::GreaterThanOrEqual => {
                    Comparison::GreaterThanOrEqual(number)
                }
                NumberComparisonSymbol::LessThanOrEqual => Comparison::LowerThanOrEqual(number),
                NumberComparisonSymbol::Equal => Comparison::Equal(number),
                NumberComparisonSymbol::NotEqual => Comparison::NotEqual(number),
            })
            .padded();

        let num_property = num_property_name
            .clone()
            .padded()
            .then(num_comparison)
            .map(|(property, cost)| QueryRestriction::NumberComparison(property, cost));

        // Text Properties
        let text_comparison_symbol = choice((
            just('=').to(TextComparisonSymbol::Equals),
            just(':').to(TextComparisonSymbol::Contains),
        ))
        .padded();

        let text_comparable = choice((
            quoted_text.map(TextComparable::String),
            regex_text.map(TextComparable::Regex),
        ))
        .padded();

        let text_comparison = text_comparison_symbol
            .then(text_comparable)
            .map(|(symbol, text)| match text {
                TextComparable::String(string) => match symbol {
                    TextComparisonSymbol::Contains => TextComparison::Contains(string),
                    TextComparisonSymbol::Equals => TextComparison::EqualTo(string),
                },
                TextComparable::Regex(regex) => TextComparison::HasMatch(regex),
            })
            .padded();

        let text_property = text_property_name
            .clone()
            .then(text_comparison)
            .map(|(property, comparison)| QueryRestriction::TextComparison(property, comparison))
            .padded();

        // Devours
        let devours_property_name = choice((keyword("devours"), keyword("dev"))).to(Text::Name);

        let null_comparison_symbol = choice((just('=').to(()), just(':').to(()))).padded();

        let devours_property = devours_property_name
            .ignore_then(null_comparison_symbol)
            .ignore_then(group_query.clone())
            .map(QueryRestriction::Devours)
            .padded();

        // Devoured by
        let devouredby_property_name =
            choice((keyword("devouredby"), keyword("deby"), keyword("dby"))).to(Text::Name);

        let devouredby_property = devouredby_property_name
            .ignore_then(null_comparison_symbol)
            .ignore_then(group_query.clone())
            .map(QueryRestriction::DevouredBy)
            .padded();

        // Fuzzy
        let fuzzy = word
            .filter(|x| x != "XOR" && x != "OR" && x != "SORT")
            .map(QueryRestriction::Fuzzy);

        // Atom
        let atom = choice((
            num_property,
            text_property,
            devours_property,
            devouredby_property,
            fuzzy,
        ))
        .padded();

        let atom = atom.or(group_restriction.clone());

        let uniop = choice((
            just('-').to(QueryOp::Not),
            just('!').to(QueryOp::LenientNot),
        ));

        let atom = uniop
            .padded()
            .repeated()
            .foldr(atom, |op, atom| match op {
                QueryOp::Not => QueryRestriction::Not(query_from_restrictions(vec![atom])),
                QueryOp::LenientNot => {
                    QueryRestriction::LenientNot(query_from_restrictions(vec![atom]))
                }
            })
            .labelled("search atom");

        let operation = choice((
            keyword("OR").to(QueryBinOp::Or),
            keyword("XOR").to(QueryBinOp::Xor),
        ));

        atom.then(operation.then(expr).or_not()).map(
            |(first, op): (QueryRestriction, Option<(QueryBinOp, QueryRestriction)>)| match op {
                None => first,
                Some((op, right)) => match op {
                    QueryBinOp::Or => QueryRestriction::Or(
                        query_from_restrictions(vec![first]),
                        query_from_restrictions(vec![right]),
                    ),
                    QueryBinOp::Xor => QueryRestriction::Xor(
                        query_from_restrictions(vec![first]),
                        query_from_restrictions(vec![right]),
                    ),
                },
            },
        )
    });

    let order = choice((
        keyword("ascending").to(Ordering::Ascending),
        keyword("descending").to(Ordering::Descending),
    ))
    .labelled("ascending or descending");

    let sort_type = choice((
        text_property_name.map(Sortable::Text),
        num_property_name.map(Sortable::Num),
    ))
    .labelled("sortable trait");

    let order =
        sort_type
            .padded()
            .then(order)
            .map(|(sort, order): (Sortable, Ordering)| match sort {
                Sortable::Text(text) => Sort::Alphabet(text, order),
                Sortable::Num(number) => Sort::Numeric(number, order),
            });

    let sort = keyword("SORT")
        .ignore_then(order)
        .labelled("sorting method")
        .or_not()
        .labelled("sorting clause or lack thereof")
        .map(|x| x.map_or(Sort::Fuzzy, |x| x));

    expr.padded()
        .repeated()
        .collect()
        .map(query_from_restrictions)
        .then(sort.padded())
        .map(|(mut query, sort)| {
            query.sort = sort;
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

#[derive(Clone)]
enum QueryOp {
    Not,
    LenientNot,
}

#[derive(Clone)]
enum QueryBinOp {
    Or,
    Xor,
}

enum Sortable {
    Text(Text),
    Num(Number),
}
