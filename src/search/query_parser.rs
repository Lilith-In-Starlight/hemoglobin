use crate::cards::{ArrayProperties, NumberProperties, StringProperties};

use super::{Comparison, Errors, Ordering, Query, QueryRestriction, Sort};

#[derive(Debug)]
enum Token {
    Word(String),
    Param(String, String),
    SuperParam(String, Vec<Token>),
    Not(Vec<Token>),
    Group(Vec<Token>),
}

enum TokenMode {
    Word,
    Param(String),
    QParam(String),
    SParam(String),
    Group,
}

#[allow(clippy::too_many_lines)]
fn tokenize_query(q: &str) -> Result<Vec<Token>, Errors> {
    let mut tokens = vec![];
    let mut word = String::new();
    let mut mode = TokenMode::Word;
    let mut paren_count = 0;
    let mut real = true;
    for ch in q.chars().chain(vec!['\n']) {
        match mode {
            TokenMode::Word => match ch {
                '-' => {
                    real = false;
                }
                '(' if word.is_empty() => {
                    mode = TokenMode::Group;
                }
                ' ' | '\n' => {
                    if !word.is_empty() {
                        if real {
                            tokens.push(Token::Word(word));
                        } else {
                            tokens.push(Token::Not(vec![Token::Word(word)]));
                        }
                    }
                    real = true;
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
                    if real {
                        tokens.push(tok);
                    } else {
                        tokens.push(Token::Not(vec![tok]));
                        real = true;
                    }
                    word = String::new();
                    mode = TokenMode::Word;
                }
                '"' if word.is_empty() => {
                    mode = TokenMode::QParam(param.to_string());
                }
                '(' if word.is_empty() => {
                    mode = TokenMode::SParam(param.to_string());
                }
                ch => word.push(ch),
            },
            TokenMode::QParam(ref param) => match ch {
                '"' => {
                    let tok = Token::Param(param.to_string(), word);
                    if real {
                        tokens.push(tok);
                    } else {
                        tokens.push(Token::Not(vec![tok]));
                        real = true;
                    }
                    word = String::new();
                    mode = TokenMode::Word;
                }
                ch => word.push(ch),
            },
            TokenMode::SParam(ref param) => match ch {
                ')' if paren_count == 0 => {
                    let tok = Token::SuperParam(param.to_string(), tokenize_query(&word)?);
                    dbg!(&tok);
                    if real {
                        tokens.push(tok);
                    } else {
                        tokens.push(Token::Not(vec![tok]));
                        real = true;
                    }
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
                    dbg!(&tok);
                    if real {
                        tokens.push(tok);
                    } else {
                        tokens.push(Token::Not(vec![tok]));
                        real = true;
                    }
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
    Ok(tokens)
}

fn parse_tokens(q: &[Token]) -> Result<Query, Errors> {
    let mut restrictions = vec![];
    let mut name = String::new();
    let mut sort = Sort::Fuzzy;
    for word in q {
        match word {
            Token::Group(group) => {
                restrictions.push(QueryRestriction::Group(parse_tokens(group)?));
            }
            Token::Word(x) => {
                name.push_str(x);
                name.push(' ');
            }
            Token::Param(param, value) => match param.as_str() {
                param @ ("sort" | "so" | "sortd" | "sod") => {
                    let order = if matches!(param, "sort" | "so") {
                        Ordering::Ascending
                    } else {
                        Ordering::Descending
                    };
                    match value.as_str() {
                        "cost" | "c" => sort = Sort::Numeric(NumberProperties::Cost, order),
                        "health" | "h" | "hp" => {
                            sort = Sort::Numeric(NumberProperties::Health, order);
                        }
                        "power" | "strength" | "damage" | "p" | "dmg" | "str" => {
                            sort = Sort::Numeric(NumberProperties::Power, order);
                        }
                        "defense" | "def" | "d" => {
                            sort = Sort::Numeric(NumberProperties::Defense, order);
                        }
                        "name" | "n" => sort = Sort::Alphabet(StringProperties::Name, order),
                        val => return Err(Errors::InvalidOrdering(val.to_owned())),
                    }
                }
                "cost" | "c" => {
                    let cmp = text_comparison_parser(value)?;
                    restrictions.push(QueryRestriction::Comparison(NumberProperties::Cost, cmp));
                }
                "health" | "h" | "hp" => {
                    let cmp = text_comparison_parser(value)?;
                    restrictions.push(QueryRestriction::Comparison(NumberProperties::Health, cmp));
                }
                "power" | "strength" | "damage" | "p" | "dmg" | "str" => {
                    let cmp = text_comparison_parser(value)?;
                    restrictions.push(QueryRestriction::Comparison(NumberProperties::Power, cmp));
                }
                "defense" | "def" | "d" => {
                    let cmp = text_comparison_parser(value)?;
                    restrictions.push(QueryRestriction::Comparison(NumberProperties::Defense, cmp));
                }
                "name" | "n" => restrictions.push(QueryRestriction::Contains(
                    StringProperties::Name,
                    value.clone(),
                )),
                "type" | "t" => restrictions.push(QueryRestriction::Contains(
                    StringProperties::Type,
                    value.clone(),
                )),
                "kin" | "k" => {
                    restrictions.push(QueryRestriction::Has(ArrayProperties::Kins, value.clone()));
                }
                "function" | "fun" | "fn" | "f" => restrictions.push(QueryRestriction::Has(
                    ArrayProperties::Functions,
                    value.clone(),
                )),
                "keyword" | "kw" => restrictions.push(QueryRestriction::HasKw(value.clone())),
                par => return Err(Errors::UnknownStringParam(par.to_owned())),
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
