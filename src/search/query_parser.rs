use regex::Regex;

use crate::{
    cards::properties::{Array, Number, Text},
    numbers::Comparison,
};

use super::{Errors, Ordering, Query, QueryRestriction, Sort, Ternary};

#[derive(Debug)]
enum Token {
    Word(String),
    Param(String, String),
    RegexParam(String, Regex),
    SuperParam(String, Vec<Token>),
    Not(Vec<Token>),
    LenientNot(Vec<Token>),
    Group(Vec<Token>),
    Or(Vec<Token>, Option<Vec<Token>>),
    Xor(Vec<Token>, Option<Vec<Token>>),
}

impl Token {
    fn polar_wrap(self, polarity: Ternary) -> Token {
        match polarity {
            Ternary::True => self,
            Ternary::False => Token::Not(vec![self]),
            Ternary::Void => Token::LenientNot(vec![self]),
        }
    }
}

enum TokenMode {
    Word,
    Param(String),
    RegexParam(String),
    QParam(String),
    SParam(String),
    Group,
}

#[derive(Default)]
struct TokenStack {
    tokens: Vec<Token>,
}

impl TokenStack {
    fn push(&mut self, token: Token) {
        match (token, self.pop()) {
            (a, None) => self.tokens.push(a),
            (a, Some(Token::Or(b, None))) => {
                self.tokens.push(Token::Or(b, Some(vec![a])));
            }
            (a, Some(Token::Xor(b, None))) => {
                self.tokens.push(Token::Xor(b, Some(vec![a])));
            }
            (a, Some(b)) => {
                self.tokens.push(b);
                self.tokens.push(a);
            }
        }
    }

    fn pop(&mut self) -> Option<Token> {
        self.tokens.pop()
    }
}

enum CharOrEnd {
    Char(char),
    End,
}

#[allow(clippy::too_many_lines)]
fn tokenize_query(q: &str) -> Result<Vec<Token>, Errors> {
    let mut tokens = TokenStack::default();
    let mut word = String::new();
    let mut mode = TokenMode::Word;
    let mut paren_count = 0;
    let mut polarity = Ternary::True;
    for ch in q.chars().map(CharOrEnd::Char).chain(vec![CharOrEnd::End]) {
        match mode {
            TokenMode::Word => match ch {
                CharOrEnd::Char('-') => match polarity {
                    Ternary::True => polarity = Ternary::False,
                    Ternary::False => polarity = Ternary::Void,
                    Ternary::Void => return Err(Errors::InvalidPolarity),
                },
                CharOrEnd::Char('(') if word.is_empty() => {
                    mode = TokenMode::Group;
                }
                CharOrEnd::Char(' ') | CharOrEnd::End => {
                    match word.as_str() {
                        "" => (),
                        "OR" => {
                            let top = tokens.pop().ok_or(Errors::InvalidOr)?;
                            tokens.push(Token::Or(vec![top], None));
                        }
                        "XOR" => {
                            let top = tokens.pop().ok_or(Errors::InvalidOr)?;
                            tokens.push(Token::Xor(vec![top], None));
                        }
                        _ => {
                            tokens.push(Token::Word(word).polar_wrap(polarity));
                        }
                    }
                    polarity = Ternary::True;
                    word = String::new();
                }
                CharOrEnd::Char(':') => {
                    if word.is_empty() {
                        return Err(Errors::AttemptedEmptyParamName);
                    }
                    mode = TokenMode::Param(word);
                    word = String::new();
                }
                CharOrEnd::Char(ch @ ('<' | '!' | '>' | '=')) => {
                    if word.is_empty() {
                        return Err(Errors::AttemptedEmptyParamName);
                    }
                    mode = TokenMode::Param(word);
                    word = String::from(ch);
                }
                CharOrEnd::Char(ch) => word.push(ch),
            },
            TokenMode::Param(ref param) => match ch {
                CharOrEnd::Char(' ') | CharOrEnd::End => {
                    let tok = Token::Param(param.to_string(), word);
                    tokens.push(tok.polar_wrap(polarity));
                    polarity = Ternary::True;
                    word = String::new();
                    mode = TokenMode::Word;
                }
                CharOrEnd::Char('"') if word.is_empty() => {
                    mode = TokenMode::QParam(param.to_string());
                }
                CharOrEnd::Char('/') if word.is_empty() => {
                    mode = TokenMode::RegexParam(param.to_string());
                }
                CharOrEnd::Char('(') if word.is_empty() => {
                    mode = TokenMode::SParam(param.to_string());
                }
                CharOrEnd::Char(ch) => word.push(ch),
            },
            TokenMode::RegexParam(ref param) => match ch {
                CharOrEnd::End | CharOrEnd::Char('/') => {
                    let tok = Token::RegexParam(
                        param.to_string(),
                        Regex::new(&word.to_lowercase()).map_err(Errors::RegexErr)?,
                    );
                    tokens.push(tok.polar_wrap(polarity));
                    polarity = Ternary::True;
                    word = String::new();
                    mode = TokenMode::Word;
                }
                CharOrEnd::Char(ch) => word.push(ch),
            },
            TokenMode::QParam(ref param) => match ch {
                CharOrEnd::Char('"') => {
                    let tok = Token::Param(param.to_string(), word);
                    tokens.push(tok.polar_wrap(polarity));
                    polarity = Ternary::True;
                    word = String::new();
                    mode = TokenMode::Word;
                }
                CharOrEnd::Char(ch) => word.push(ch),
                CharOrEnd::End => return Err(Errors::UnclosedString),
            },
            TokenMode::SParam(ref param) => match ch {
                CharOrEnd::Char(')') if paren_count == 0 => {
                    let tok = Token::SuperParam(param.to_string(), tokenize_query(&word)?);
                    tokens.push(tok.polar_wrap(polarity));
                    polarity = Ternary::True;
                    word = String::new();
                    mode = TokenMode::Word;
                }
                CharOrEnd::Char(ch @ '(') => {
                    paren_count += 1;
                    word.push(ch);
                }
                CharOrEnd::Char(ch @ ')') if paren_count > 0 => {
                    paren_count -= 1;
                    word.push(ch);
                }
                CharOrEnd::Char(ch) => word.push(ch),
                CharOrEnd::End => return Err(Errors::UnclosedRegex),
            },
            TokenMode::Group => match ch {
                CharOrEnd::Char(')') if paren_count == 0 => {
                    let tok = Token::Group(tokenize_query(&word)?);
                    tokens.push(tok.polar_wrap(polarity));
                    polarity = Ternary::True;
                    word = String::new();
                    mode = TokenMode::Word;
                }
                CharOrEnd::Char(ch @ '(') => {
                    paren_count += 1;
                    word.push(ch);
                }
                CharOrEnd::Char(ch @ ')') if paren_count > 0 => {
                    paren_count -= 1;
                    word.push(ch);
                }
                CharOrEnd::Char(ch) => word.push(ch),
                CharOrEnd::End => return Err(Errors::UnclosedSubquery),
            },
        }
    }
    Ok(tokens.tokens)
}

fn parse_tokens(q: &[Token]) -> Result<Query, Errors> {
    let mut restrictions = vec![];
    let mut name = String::new();
    let mut sort = Sort::Fuzzy;
    for word in q {
        match word {
            Token::RegexParam(field, regex) => match get_property_from_name(field.as_str())? {
                Properties::StringProperty(property) => {
                    restrictions.push(QueryRestriction::Regex(property, regex.clone()));
                }
                _ => return Err(Errors::NonRegexable(field.clone())),
            },
            Token::Or(group1, group2) => match group2 {
                None => return Err(Errors::InvalidOr),
                Some(group2) => {
                    let mut group1 = parse_tokens(group1)?;
                    group1.sort = Sort::None;
                    let mut group2 = parse_tokens(group2)?;
                    group2.sort = Sort::None;
                    restrictions.push(QueryRestriction::Or(group1, group2));
                }
            },
            Token::Xor(group1, group2) => match group2 {
                None => return Err(Errors::InvalidOr),
                Some(group2) => {
                    let mut group1 = parse_tokens(group1)?;
                    group1.sort = Sort::None;
                    let mut group2 = parse_tokens(group2)?;
                    group2.sort = Sort::None;
                    restrictions.push(QueryRestriction::Xor(group1, group2));
                }
            },
            Token::Group(group) => {
                let mut group = parse_tokens(group)?;
                group.sort = Sort::None;
                restrictions.push(QueryRestriction::Group(group));
            }
            Token::Word(x) => {
                name.push_str(x);
                name.push(' ');
            }
            Token::Param(param, value) => match get_property_from_name(param)? {
                Properties::Sort(order) => match get_property_from_name(value)? {
                    Properties::NumProperty(property) => sort = Sort::Numeric(property, order),
                    Properties::StringProperty(property) => {
                        sort = Sort::Alphabet(property, order);
                    }

                    _ => return Err(Errors::NotSortable),
                },
                Properties::NumProperty(property) => {
                    let cmp = text_comparison_parser(value)?;
                    restrictions.push(QueryRestriction::Comparison(property, cmp));
                }
                Properties::StringProperty(property) => {
                    restrictions.push(QueryRestriction::Contains(property, value.clone()));
                }
                Properties::ArrayProperty(property) => {
                    restrictions.push(QueryRestriction::Has(property, value.clone()));
                }
                Properties::Keywords => restrictions.push(QueryRestriction::HasKw(value.clone())),
            },
            Token::SuperParam(param, value) => match param.as_str() {
                "devours" | "dev" | "de" | "devs" => {
                    let mut parsed_subquery = parse_tokens(value)?;
                    parsed_subquery.sort = Sort::None;
                    restrictions.push(QueryRestriction::Devours(parsed_subquery));
                }
                "devouredby" | "devby" | "deby" | "dby" | "db" => {
                    let mut parsed_subquery = parse_tokens(value)?;
                    parsed_subquery.sort = Sort::None;
                    restrictions.push(QueryRestriction::DevouredBy(parsed_subquery));
                    // devoured_by = Some(Box::new(parsed_subquery));
                }
                par => return Err(Errors::UnknownSubQueryParam(par.to_owned())),
            },
            Token::Not(tokens) => {
                let mut group = parse_tokens(tokens)?;
                group.sort = Sort::None;
                restrictions.push(QueryRestriction::Not(group));
            }
            Token::LenientNot(tokens) => {
                let mut group = parse_tokens(tokens)?;
                group.sort = Sort::None;
                restrictions.push(QueryRestriction::LenientNot(group));
            }
        }
    }
    let name = name.trim().to_string();
    if !name.is_empty() {
        restrictions.push(QueryRestriction::Fuzzy(name.clone()));
    }
    Ok(Query {
        name,
        restrictions,
        sort,
    })
}

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
        "defense" | "def" | "d" => Ok(Properties::NumProperty(Number::Defense)),
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

/// A parser for string search queries.
/// # Errors
/// Whenever a query cannot be parsed
pub fn query_parser(q: &str) -> Result<Query, Errors> {
    let q = tokenize_query(q)?;
    parse_tokens(&q)
}

pub(crate) fn text_comparison_parser(s: &str) -> Result<Comparison, Errors> {
    match s.parse::<usize>() {
        Ok(x) => Ok(Comparison::Equal(x)),
        Err(_) => {
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
        }
    }
}
