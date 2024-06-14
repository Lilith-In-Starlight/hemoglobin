use crate::cards::Card;

use super::{Comparison, Errors, QueryRestriction};

#[derive(Debug)]
enum Token {
    Word(String),
    Param(String, String),
    SuperParam(String, Vec<Token>),
    Not(Vec<Token>),
}

enum TokenMode {
    Word,
    Param(String),
    QParam(String),
    SParam(String),
}

fn tokenize_query(q: &str) -> Result<Vec<Token>, Errors> {
    let mut tokens = vec![];
    let mut word = String::new();
    let mut mode = TokenMode::Word;
    let mut paren_count = 0;
    let mut real = false;
    for ch in q.chars().chain(vec!['\n']) {
        match mode {
            TokenMode::Word => match ch {
                '-' => {
                    real = true;
                }
                ' ' | '\n' => {
                    if real {
                        tokens.push(Token::Word(word));
                    } else {
                        tokens.push(Token::Not(vec![Token::Word(word)]));
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
                '{' if word.is_empty() => {
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

fn parse_tokens(q: &[Token]) -> Result<Vec<QueryRestriction>, Errors> {
    let mut restrictions = vec![];
    let mut string = String::new();
    for word in q {
        match word {
            Token::Word(x) => {
                string.push_str(x);
                string.push(' ');
            }
            Token::Param(param, value) => match param.as_str() {
                "cost" | "c" => {
                    let cmp = text_comparison_parser(value)?;
                    restrictions.push(QueryRestriction::Comparison(Box::new(Card::get_cost), cmp));
                }
                "health" | "h" | "hp" => {
                    let cmp = text_comparison_parser(value)?;
                    restrictions.push(QueryRestriction::Comparison(
                        Box::new(Card::get_health),
                        cmp,
                    ));
                }
                "power" | "strength" | "damage" | "p" | "dmg" | "str" => {
                    let cmp = text_comparison_parser(value)?;
                    restrictions.push(QueryRestriction::Comparison(Box::new(Card::get_power), cmp));
                }
                "defense" | "def" | "d" => {
                    let cmp = text_comparison_parser(value)?;
                    restrictions.push(QueryRestriction::Comparison(
                        Box::new(Card::get_defense),
                        cmp,
                    ));
                }
                "name" | "n" => restrictions.push(QueryRestriction::Contains(
                    Box::new(Card::get_name),
                    value.clone(),
                )),
                "type" | "t" => restrictions.push(QueryRestriction::Contains(
                    Box::new(Card::get_type),
                    value.clone(),
                )),
                "kin" | "k" => restrictions.push(QueryRestriction::Has(
                    Box::new(Card::get_kins),
                    value.clone(),
                )),
                "function" | "fun" | "fn" | "f" => restrictions.push(QueryRestriction::Has(
                    Box::new(Card::get_functions),
                    value.clone(),
                )),
                "keyword" | "kw" => restrictions.push(QueryRestriction::HasKw(
                    Box::new(Card::get_keywords),
                    value.clone(),
                )),
                _ => return Err(Errors::UnknownParam),
            },
            Token::SuperParam(param, value) => {
                if param == "devour" {
                    todo!();
                }
            }
            Token::Not(tokens) => {
                restrictions.push(QueryRestriction::Not(parse_tokens(tokens)?));
            }
        }
    }
    let string = string.trim().to_string();
    restrictions.push(QueryRestriction::Fuzzy(string));
    Ok(restrictions)
}

/// # Errors
/// Whenever a query cannot be parsed
pub fn query_parser(q: &str) -> Result<Vec<QueryRestriction>, Errors> {
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
