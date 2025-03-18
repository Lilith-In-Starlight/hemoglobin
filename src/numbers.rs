pub mod compare;
pub mod imprecise_ord;
use std::{
    cmp::Ordering,
    fmt::{Debug, Display},
    num::TryFromIntError,
};

use serde::{
    de::{Error, Visitor},
    Deserialize, Deserializer, Serialize,
};

use crate::search::{query_parser::text_comparison_parser, Ternary};

/// Represents a number that may match a range instead of a single number
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

/// Represents a Bloodless Number. Bloodless Numbers are defined in section 1.7 of the Bloodless Abstract Rules.
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
    /// If a number is a variable, it will usually be assumed to be zero. This might change in the future.
    #[must_use]
    pub const fn assume(&self) -> usize {
        match self {
            Self::Const(x) => *x,
            Self::Var(_) => 0,
        }
    }
}

/// A trait for types that can be numerically matched with `usize`
pub trait Compare {
    fn gt(&self, comparison: usize) -> Ternary;
    fn gt_eq(&self, comparison: usize) -> Ternary;
    fn lt(&self, comparison: usize) -> Ternary;
    fn lt_eq(&self, comparison: usize) -> Ternary;
    fn eq(&self, comparison: usize) -> Ternary;
    fn ne(&self, comparison: usize) -> Ternary;
}

/// A version of ordering that works over ranges and does not necessitate a notion of equality, which cannot be defined for Bloodless number ranges
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
        deserialize_maybe_var::<D>(deserializer)
    }
}

impl<'de> Deserialize<'de> for MaybeImprecise {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MiVisitor;
        impl Visitor<'_> for MiVisitor {
            type Value = MaybeImprecise;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("single character string or number")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                str_as_maybe_var(v).map_or_else(
                    || {
                        text_comparison_parser(v).map_or_else(
                            |_| Err(Error::custom("expected a bloodless number or a comparison")),
                            |x| Ok(Self::Value::Imprecise(x)),
                        )
                    },
                    |value| Ok(MaybeImprecise::Precise(value)),
                )
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                u64_as_maybe_var(v).map(MaybeImprecise::Precise).map_err(|_| {
                    E::custom(
                        "converted from a value greater than the current architecture's pointer size",
                    )
                })
            }
        }
        deserializer.deserialize_any(MiVisitor)
    }
}

fn deserialize_maybe_var<'de, D: Deserializer<'de>>(deserializer: D) -> Result<MaybeVar, D::Error> {
    struct MvVisitor;
    impl Visitor<'_> for MvVisitor {
        type Value = MaybeVar;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("single character string or number")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            match v.chars().next() {
                Some(char) if char.is_alphabetic() => Ok(MaybeVar::Var(char)),
                _ => Err(Error::custom(
                    "numbers can only be single letters or integers",
                )),
            }
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            u64_as_maybe_var(v).map_err(|_| {
                E::custom(
                    "converted from a value greater than the current architecture's pointer size",
                )
            })
        }
    }

    deserializer.deserialize_any(MvVisitor)
}

fn u64_as_maybe_var(v: u64) -> Result<MaybeVar, TryFromIntError> {
    match v.try_into() {
        Ok(number) => Ok(MaybeVar::Const(number)),
        Err(x) => Err(x),
    }
}

fn str_as_maybe_var(v: &str) -> Option<MaybeVar> {
    v.chars()
        .next()
        .filter(|x| x.is_alphabetic())
        .map(MaybeVar::Var)
}
