pub mod imprecise_eq;
pub mod imprecise_ord;
use std::{cmp::Ordering, fmt::Display};

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::QueryMatch;

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
    pub fn compare<T: PartialOrd<usize>>(&self, a: &T) -> bool {
        match self {
            Comparison::GreaterThan(x) => a > x,
            Comparison::Equal(x) => a == x,
            Comparison::LowerThan(x) => a < x,
            Comparison::NotEqual(x) => a != x,
            Comparison::GreaterThanOrEqual(x) => a >= x,
            Comparison::LowerThanOrEqual(x) => a <= x,
        }
    }

    pub fn maybe_compare<T>(&self, a: Option<T>) -> QueryMatch
    where
        Comparison: ImpreciseOrd<T>,
    {
        match a {
            Some(a) => {
                if self.imprecise_cmp(&a).is_eq() {
                    QueryMatch::Match
                } else {
                    QueryMatch::NotMatch
                }
            }
            None => QueryMatch::NotHave,
        }
    }
}

impl<'de> Deserialize<'de> for MaybeVar {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        if let Some(text) = value.as_str() {
            return Ok(MaybeVar::Var(text.chars().next().unwrap()));
        }
        if let Some(text) = value.as_u64() {
            return Ok(MaybeVar::Const(text.try_into().unwrap()));
        }
        Ok(MaybeVar::default())
    }
}

impl<'de> Deserialize<'de> for MaybeImprecise {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        if let Some(text) = value.as_str() {
            return Ok(MaybeImprecise::Precise(MaybeVar::Var(
                text.chars().next().unwrap(),
            )));
        }
        if let Some(text) = value.as_u64() {
            return Ok(MaybeImprecise::Precise(MaybeVar::Const(
                text.try_into().unwrap(),
            )));
        }
        Ok(MaybeImprecise::Precise(MaybeVar::default()))
    }
}
