use regex::Regex;

use crate::{
    cards::{ArrayProperties, NumberProperties, StringProperties},
    QueryMatch,
};

use super::{Comparison, Errors, Ordering, Query, QueryRestriction, Sort};

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
    fn polar_wrap(self, polarity: QueryMatch) -> Token {
        match polarity {
            QueryMatch::Match => self,
            QueryMatch::NotMatch => Token::Not(vec![self]),
            QueryMatch::NotHave => Token::LenientNot(vec![self]),
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

#[allow(clippy::too_many_lines)]
fn tokenize_query(q: &str) -> Result<Vec<Token>, Errors> {
    let mut tokens = TokenStack::default();
    let mut word = String::new();
    let mut mode = TokenMode::Word;
    let mut paren_count = 0;
    let mut polarity = QueryMatch::Match;
    for ch in q.chars().chain(vec!['\n']) {
        match mode {
            TokenMode::Word => match ch {
                '-' => match polarity {
                    QueryMatch::Match => polarity = QueryMatch::NotMatch,
                    QueryMatch::NotMatch => polarity = QueryMatch::NotHave,
                    QueryMatch::NotHave => return Err(Errors::InvalidPolarity),
                },
                '(' if word.is_empty() => {
                    mode = TokenMode::Group;
                }
                ' ' | '\n' => {
                    if !word.is_empty() {
                        match word.as_str() {
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
                    }
                    polarity = QueryMatch::Match;
                    word = String::new();
                }
                ':' => {
                    mode = TokenMode::Param(word);
                    word = String::new();
                }
                '<' | '!' | '>' | '=' => {
                    mode = TokenMode::Param(word);
                    word = String::from(ch);
                }
                ch => word.push(ch),
            },
            TokenMode::Param(ref param) => match ch {
                ' ' | '\n' => {
                    let tok = Token::Param(param.to_string(), word);
                    tokens.push(tok.polar_wrap(polarity));
                    polarity = QueryMatch::Match;
                    word = String::new();
                    mode = TokenMode::Word;
                }
                '"' if word.is_empty() => {
                    mode = TokenMode::QParam(param.to_string());
                }
                '/' if word.is_empty() => {
                    mode = TokenMode::RegexParam(param.to_string());
                }
                '(' if word.is_empty() => {
                    mode = TokenMode::SParam(param.to_string());
                }
                ch => word.push(ch),
            },
            TokenMode::RegexParam(ref param) => match ch {
                '\n' | '/' => {
                    let tok = Token::RegexParam(
                        param.to_string(),
                        Regex::new(&word.to_lowercase()).map_err(Errors::RegexErr)?,
                    );
                    tokens.push(tok.polar_wrap(polarity));
                    polarity = QueryMatch::Match;
                    word = String::new();
                    mode = TokenMode::Word;
                }
                ch => word.push(ch),
            },
            TokenMode::QParam(ref param) => match ch {
                '"' => {
                    let tok = Token::Param(param.to_string(), word);
                    tokens.push(tok.polar_wrap(polarity));
                    polarity = QueryMatch::Match;
                    word = String::new();
                    mode = TokenMode::Word;
                }
                ch => word.push(ch),
            },
            TokenMode::SParam(ref param) => match ch {
                ')' if paren_count == 0 => {
                    let tok = Token::SuperParam(param.to_string(), tokenize_query(&word)?);
                    tokens.push(tok.polar_wrap(polarity));
                    polarity = QueryMatch::Match;
                    word = String::new();
                    mode = TokenMode::Word;
                }
                '(' => {
                    paren_count += 1;
                    word.push(ch);
                }
                ')' if paren_count > 0 => {
                    paren_count -= 1;
                    word.push(ch);
                }
                ch => word.push(ch),
            },
            TokenMode::Group => match ch {
                ')' if paren_count == 0 => {
                    let tok = Token::Group(tokenize_query(&word)?);
                    tokens.push(tok.polar_wrap(polarity));
                    polarity = QueryMatch::Match;
                    word = String::new();
                    mode = TokenMode::Word;
                }
                '(' => {
                    paren_count += 1;
                    word.push(ch);
                }
                ')' if paren_count > 0 => {
                    paren_count -= 1;
                    word.push(ch);
                }
                ch => word.push(ch),
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
                Some(group2) => restrictions.push(QueryRestriction::Or(
                    parse_tokens(group1)?,
                    parse_tokens(group2)?,
                )),
            },
            Token::Xor(group1, group2) => match group2 {
                None => return Err(Errors::InvalidOr),
                Some(group2) => restrictions.push(QueryRestriction::Xor(
                    parse_tokens(group1)?,
                    parse_tokens(group2)?,
                )),
            },
            Token::Group(group) => {
                restrictions.push(QueryRestriction::Group(parse_tokens(group)?));
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
                    restrictions.push(QueryRestriction::Devours(parse_tokens(value)?));
                }
                par => return Err(Errors::UnknownSubQueryParam(par.to_owned())),
            },
            Token::Not(tokens) => {
                restrictions.push(QueryRestriction::Not(parse_tokens(tokens)?));
            }
            Token::LenientNot(tokens) => {
                restrictions.push(QueryRestriction::LenientNot(parse_tokens(tokens)?));
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
        "name" | "n" => Ok(Properties::StringProperty(StringProperties::Name)),
        "type" | "t" => Ok(Properties::StringProperty(StringProperties::Type)),
        "cost" | "c" => Ok(Properties::NumProperty(NumberProperties::Cost)),
        "health" | "h" | "hp" => Ok(Properties::NumProperty(NumberProperties::Health)),
        "power" | "strength" | "damage" | "p" | "dmg" | "str" => {
            Ok(Properties::NumProperty(NumberProperties::Power))
        }
        "defense" | "def" | "d" => Ok(Properties::NumProperty(NumberProperties::Defense)),
        "kin" | "k" => Ok(Properties::ArrayProperty(ArrayProperties::Kins)),
        "function" | "fun" | "fn" | "f" => {
            Ok(Properties::ArrayProperty(ArrayProperties::Functions))
        }
        "keyword" | "kw" => Ok(Properties::Keywords),
        "sort" | "so" => Ok(Properties::Sort(Ordering::Ascending)),
        "sortd" | "sod" => Ok(Properties::Sort(Ordering::Descending)),
        _ => Err(Errors::UnknownStringParam(str.to_owned())),
    }
}

pub enum Properties {
    NumProperty(NumberProperties),
    StringProperty(StringProperties),
    ArrayProperty(ArrayProperties),
    Sort(Ordering),
    Keywords,
}

/// # Errors
/// Whenever a query cannot be parsed
pub fn query_parser(q: &str) -> Result<Query, Errors> {
    let q = tokenize_query(q)?;
    parse_tokens(&q)
}

fn text_comparison_parser(s: &str) -> Result<Comparison, Errors> {
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
