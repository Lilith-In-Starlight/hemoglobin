pub mod compare;
pub mod imprecise_eq;
pub mod imprecise_ord;
use std::{cmp::Ordering, fmt::Display};

use serde::{de::Error, Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::search::{query_parser::text_comparison_parser, Ternary};

#[derive(Serialize, Debug, PartialEq, Clone)]
pub enum MaybeImprecise {
    Precise(MaybeVar),
    Imprecise(Comparison),
}

impl MaybeImprecise {
    #[must_use]
    pub fn as_comparison(&self) -> Comparison {
        match self {
            Self::Precise(x) => Comparison::Equal(x.assume()),
            Self::Imprecise(x) => *x,
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub enum MaybeVar {
    Const(usize),
    Var(char),
}

impl Display for MaybeVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Const(x) => write!(f, "{x}"),
            Self::Var(x) => write!(f, "{x}"),
        }
    }
}

impl Default for MaybeVar {
    fn default() -> Self {
        Self::Const(0)
    }
}

impl MaybeVar {
    #[must_use]
    pub fn as_maybe_imprecise(&self) -> MaybeImprecise {
        MaybeImprecise::Precise(self.clone())
    }
    #[must_use]
    pub fn assume(&self) -> usize {
        match self {
            Self::Const(x) => *x,
            Self::Var(_) => 0,
        }
    }
}

pub trait Compare {
    fn gt(&self, comparison: usize) -> Ternary;
    fn gt_eq(&self, comparison: usize) -> Ternary;
    fn lt(&self, comparison: usize) -> Ternary;
    fn lt_eq(&self, comparison: usize) -> Ternary;
    fn eq(&self, comparison: usize) -> Ternary;
    fn ne(&self, comparison: usize) -> Ternary;
}

pub trait ImpreciseEq<Other> {
    fn imprecise_eq(&self, other: &Other) -> bool;
}

pub trait ImpreciseOrd<Other> {
    fn imprecise_cmp(&self, other: &Other) -> Ordering;
}

/// Comparisons to a certain numeric value
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Comparison {
    GreaterThan(usize),
    GreaterThanOrEqual(usize),
    LowerThanOrEqual(usize),
    Equal(usize),
    LowerThan(usize),
    NotEqual(usize),
}

impl Display for Comparison {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GreaterThan(number) => write!(f, "greater than {number}"),
            Self::GreaterThanOrEqual(number) => write!(f, "greater than or equal to {number}"),
            Self::LowerThanOrEqual(number) => write!(f, "lower than than or equal to {number}"),
            Self::Equal(number) => write!(f, "equal to {number}"),
            Self::LowerThan(number) => write!(f, "lower than {number}"),
            Self::NotEqual(number) => write!(f, "other than {number}"),
        }
    }
}

impl Comparison {
    pub fn compare<T: Compare>(&self, a: &T) -> Ternary {
        match self {
            Comparison::GreaterThan(x) => a.gt(*x),
            Comparison::Equal(x) => a.eq(*x),
            Comparison::LowerThan(x) => a.lt(*x),
            Comparison::NotEqual(x) => a.ne(*x),
            Comparison::GreaterThanOrEqual(x) => a.gt_eq(*x),
            Comparison::LowerThanOrEqual(x) => a.lt_eq(*x),
        }
    }
}

impl<'de> Deserialize<'de> for MaybeVar {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        deserialize_maybe_var::<D>(&value)
    }
}

impl<'de> Deserialize<'de> for MaybeImprecise {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        if let Ok(x) = deserialize_maybe_var::<D>(&value) {
            Ok(MaybeImprecise::Precise(x))
        } else {
            if let Some(text) = value.as_str() {
                let comparison =
                    text_comparison_parser(text).map_err(|x| Error::custom(format!("{x:#?}")))?;
                return Ok(MaybeImprecise::Imprecise(comparison));
            }
            Err(Error::custom(
                "Numbers that might be imprecise must be integers, single letters, or comparisons",
            ))
        }
    }
}

fn deserialize_maybe_var<'de, D: Deserializer<'de>>(value: &Value) -> Result<MaybeVar, D::Error> {
    if let Some(text) = value.as_str() {
        match text.chars().next() {
            Some(char) if char.is_alphabetic() => return Ok(MaybeVar::Var(char)),
            _ => {
                return Err(Error::custom(
                    "Numbers can only be single letters or integers",
                ))
            }
        }
    }
    if let Some(text) = value.as_u64() {
        match text.try_into() {
            Ok(number) => return Ok(MaybeVar::Const(number)),
            Err(err) => {
                return Err(Error::custom(format!(
                    "Error while turning u64 into usize for a number: {err}"
                )))
            }
        }
    }
    Err(Error::custom(
        "Numbers can only be single letters or integers",
    ))
}
