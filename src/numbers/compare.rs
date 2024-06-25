use crate::QueryMatch;

use super::{Compare, Comparison, MaybeImprecise};

impl<T: Compare> Compare for Option<T> {
    fn gt(&self, comparison: usize) -> QueryMatch {
        match self {
            None => QueryMatch::NotHave,
            Some(x) => x.gt(comparison),
        }
    }

    fn gt_eq(&self, comparison: usize) -> QueryMatch {
        match self {
            None => QueryMatch::NotHave,
            Some(x) => x.gt_eq(comparison),
        }
    }

    fn lt(&self, comparison: usize) -> QueryMatch {
        match self {
            None => QueryMatch::NotHave,
            Some(x) => x.lt(comparison),
        }
    }

    fn lt_eq(&self, comparison: usize) -> QueryMatch {
        match self {
            None => QueryMatch::NotHave,
            Some(x) => x.lt_eq(comparison),
        }
    }

    fn eq(&self, comparison: usize) -> QueryMatch {
        match self {
            None => QueryMatch::NotHave,
            Some(x) => x.eq(comparison),
        }
    }

    fn ne(&self, comparison: usize) -> QueryMatch {
        match self {
            None => QueryMatch::NotHave,
            Some(x) => x.ne(comparison),
        }
    }
}

impl Compare for MaybeImprecise {
    fn gt(&self, comparison: usize) -> crate::QueryMatch {
        match self {
            MaybeImprecise::Precise(x) => (x.assume() > comparison).into(),
            MaybeImprecise::Imprecise(x) => match x {
                Comparison::Equal(x) => (*x > comparison).into(),
                Comparison::GreaterThan(_)
                | Comparison::GreaterThanOrEqual(_)
                | Comparison::NotEqual(_) => QueryMatch::Match,
                Comparison::LowerThan(x) => (*x > comparison + 1).into(),
                Comparison::LowerThanOrEqual(x) => (*x > comparison).into(),
            },
        }
    }

    fn gt_eq(&self, comparison: usize) -> crate::QueryMatch {
        match self {
            MaybeImprecise::Precise(x) => (x.assume() >= comparison).into(),
            MaybeImprecise::Imprecise(x) => match x {
                Comparison::Equal(x) => (*x >= comparison).into(),
                Comparison::GreaterThan(_)
                | Comparison::GreaterThanOrEqual(_)
                | Comparison::NotEqual(_) => QueryMatch::Match,
                Comparison::LowerThan(x) => (*x > comparison).into(),
                Comparison::LowerThanOrEqual(x) => (*x > comparison).into(),
            },
        }
    }

    fn lt(&self, comparison: usize) -> crate::QueryMatch {
        match self {
            MaybeImprecise::Precise(x) => (x.assume() < comparison).into(),
            MaybeImprecise::Imprecise(x) => match x {
                Comparison::Equal(x) => (*x < comparison).into(),
                Comparison::GreaterThan(x) => (*x - 1 < comparison).into(),
                Comparison::GreaterThanOrEqual(x) => (*x < comparison).into(),
                Comparison::LowerThan(_) => QueryMatch::Match,
                Comparison::LowerThanOrEqual(_) => QueryMatch::Match,
                Comparison::NotEqual(_) => QueryMatch::Match,
            },
        }
    }

    fn lt_eq(&self, comparison: usize) -> crate::QueryMatch {
        match self {
            MaybeImprecise::Precise(x) => (x.assume() <= comparison).into(),
            MaybeImprecise::Imprecise(x) => match x {
                Comparison::Equal(x) => (*x <= comparison).into(),
                Comparison::GreaterThan(x) => (*x < comparison).into(),
                Comparison::GreaterThanOrEqual(x) => (*x < comparison).into(),
                Comparison::LowerThan(_) => QueryMatch::Match,
                Comparison::LowerThanOrEqual(_) => QueryMatch::Match,
                Comparison::NotEqual(_) => QueryMatch::Match,
            },
        }
    }

    fn eq(&self, comparison: usize) -> crate::QueryMatch {
        match self {
            MaybeImprecise::Precise(x) => (comparison == x.assume()).into(),
            MaybeImprecise::Imprecise(x) => match x {
                Comparison::Equal(x) => (comparison == *x).into(),
                Comparison::GreaterThan(x) => (comparison < *x).into(),
                Comparison::GreaterThanOrEqual(x) => (comparison <= *x).into(),
                Comparison::LowerThan(x) => (comparison > *x).into(),
                Comparison::LowerThanOrEqual(x) => (comparison >= *x).into(),
                Comparison::NotEqual(x) => (comparison != *x).into(),
            },
        }
    }

    fn ne(&self, comparison: usize) -> crate::QueryMatch {
        match self {
            MaybeImprecise::Precise(x) => (comparison != x.assume()).into(),
            MaybeImprecise::Imprecise(x) => match x {
                Comparison::Equal(x) => (comparison != *x).into(),
                Comparison::GreaterThan(_) => QueryMatch::Match,
                Comparison::GreaterThanOrEqual(_) => QueryMatch::Match,
                Comparison::LowerThan(_) => QueryMatch::Match,
                Comparison::LowerThanOrEqual(_) => QueryMatch::Match,
                Comparison::NotEqual(_) => QueryMatch::Match,
            },
        }
    }
}

impl Compare for usize {
    fn gt(&self, comparison: usize) -> crate::QueryMatch {
        (*self > comparison).into()
    }

    fn gt_eq(&self, comparison: usize) -> crate::QueryMatch {
        (*self >= comparison).into()
    }

    fn lt(&self, comparison: usize) -> crate::QueryMatch {
        (*self < comparison).into()
    }

    fn lt_eq(&self, comparison: usize) -> crate::QueryMatch {
        (*self <= comparison).into()
    }

    fn eq(&self, comparison: usize) -> crate::QueryMatch {
        (*self == comparison).into()
    }

    fn ne(&self, comparison: usize) -> crate::QueryMatch {
        (*self != comparison).into()
    }
}
