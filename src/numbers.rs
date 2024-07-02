pub mod compare;
pub mod imprecise_eq;
pub mod imprecise_ord;
use std::{
    cmp::Ordering,
    fmt::{Debug, Display},
};

use serde::{de::Error, Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::search::{query_parser::text_comparison_parser, Ternary};

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum MaybeImprecise {
    Precise(MaybeVar),
    Imprecise(Comparison),
}

impl Default for MaybeImprecise {
    fn default() -> Self {
        Self::Precise(MaybeVar::Const(0))
    }
}

impl Display for MaybeImprecise {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Precise(x) => write!(f, "{x}"),
            Self::Imprecise(x) => write!(f, "{x}"),
        }
    }
}

impl MaybeImprecise {
    #[must_use]
    pub const fn as_comparison(&self) -> Comparison {
        match self {
            Self::Precise(x) => Comparison::Equal(x.assume()),
            Self::Imprecise(x) => *x,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub const fn assume(&self) -> usize {
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
#[derive(Eq, PartialEq, Serialize, Deserialize, Debug, Clone, Copy)]
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
            Self::GreaterThan(number) => write!(f, "> {number}"),
            Self::GreaterThanOrEqual(number) => write!(f, ">= {number}"),
            Self::LowerThanOrEqual(number) => write!(f, "<= {number}"),
            Self::Equal(number) => write!(f, "= {number}"),
            Self::LowerThan(number) => write!(f, "< {number}"),
            Self::NotEqual(number) => write!(f, "!= {number}"),
        }
    }
}

impl Comparison {
    pub fn compare<T: Compare + Debug>(&self, a: &T) -> Ternary {
        match self {
            Self::GreaterThan(x) => a.gt(*x),
            Self::Equal(x) => a.eq(*x),
            Self::LowerThan(x) => a.lt(*x),
            Self::NotEqual(x) => a.ne(*x),
            Self::GreaterThanOrEqual(x) => a.gt_eq(*x),
            Self::LowerThanOrEqual(x) => a.lt_eq(*x),
        }
    }
}

impl Serialize for MaybeVar {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Const(x) => serializer.serialize_u64((*x).try_into().unwrap()),
            Self::Var(x) => serializer.serialize_str(&x.to_string()),
        }
    }
}

impl Serialize for MaybeImprecise {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Precise(x) => MaybeVar::serialize(x, serializer),
            Self::Imprecise(x) => match x {
                Comparison::Equal(x) => serializer.serialize_u64((*x).try_into().unwrap()),
                Comparison::GreaterThan(x) => serializer.serialize_str(&format!(">{x}")),
                Comparison::GreaterThanOrEqual(x) => serializer.serialize_str(&format!(">={x}")),
                Comparison::LowerThan(x) => serializer.serialize_str(&format!("<{x}")),
                Comparison::LowerThanOrEqual(x) => serializer.serialize_str(&format!("<={x}")),
                Comparison::NotEqual(x) => serializer.serialize_str(&format!("!={x}")),
            },
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
            Ok(Self::Precise(x))
        } else {
            if let Some(text) = value.as_str() {
                let comparison =
                    text_comparison_parser(text).map_err(|x| Error::custom(format!("{x:#?}")))?;
                return Ok(Self::Imprecise(comparison));
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
