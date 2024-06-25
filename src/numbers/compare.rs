use crate::search::Ternary;

use super::{Compare, Comparison, MaybeImprecise};

impl<T: Compare> Compare for Option<T> {
    fn gt(&self, comparison: usize) -> Ternary {
        match self {
            None => Ternary::Void,
            Some(x) => x.gt(comparison),
        }
    }

    fn gt_eq(&self, comparison: usize) -> Ternary {
        match self {
            None => Ternary::Void,
            Some(x) => x.gt_eq(comparison),
        }
    }

    fn lt(&self, comparison: usize) -> Ternary {
        match self {
            None => Ternary::Void,
            Some(x) => x.lt(comparison),
        }
    }

    fn lt_eq(&self, comparison: usize) -> Ternary {
        match self {
            None => Ternary::Void,
            Some(x) => x.lt_eq(comparison),
        }
    }

    fn eq(&self, comparison: usize) -> Ternary {
        match self {
            None => Ternary::Void,
            Some(x) => x.eq(comparison),
        }
    }

    fn ne(&self, comparison: usize) -> Ternary {
        match self {
            None => Ternary::Void,
            Some(x) => x.ne(comparison),
        }
    }
}

impl Compare for MaybeImprecise {
    fn gt(&self, comparison: usize) -> Ternary {
        match self {
            MaybeImprecise::Precise(x) => (x.assume() > comparison).into(),
            MaybeImprecise::Imprecise(x) => match x {
                Comparison::Equal(x) => (*x > comparison).into(),
                Comparison::GreaterThan(_)
                | Comparison::GreaterThanOrEqual(_)
                | Comparison::NotEqual(_) => Ternary::True,
                Comparison::LowerThan(x) => (*x > comparison + 1).into(),
                Comparison::LowerThanOrEqual(x) => (*x > comparison).into(),
            },
        }
    }

    fn gt_eq(&self, comparison: usize) -> Ternary {
        match self {
            MaybeImprecise::Precise(x) => (x.assume() >= comparison).into(),
            MaybeImprecise::Imprecise(x) => match x {
                Comparison::Equal(x) => (*x >= comparison).into(),
                Comparison::GreaterThan(_)
                | Comparison::GreaterThanOrEqual(_)
                | Comparison::NotEqual(_) => Ternary::True,
                Comparison::LowerThan(x) => (*x > comparison).into(),
                Comparison::LowerThanOrEqual(x) => (*x > comparison).into(),
            },
        }
    }

    fn lt(&self, comparison: usize) -> Ternary {
        match self {
            MaybeImprecise::Precise(x) => (x.assume() < comparison).into(),
            MaybeImprecise::Imprecise(x) => match x {
                Comparison::Equal(x) => (*x < comparison).into(),
                Comparison::GreaterThan(x) => (*x - 1 < comparison).into(),
                Comparison::GreaterThanOrEqual(x) => (*x < comparison).into(),
                Comparison::LowerThan(_) => Ternary::True,
                Comparison::LowerThanOrEqual(_) => Ternary::True,
                Comparison::NotEqual(_) => Ternary::True,
            },
        }
    }

    fn lt_eq(&self, comparison: usize) -> Ternary {
        match self {
            MaybeImprecise::Precise(x) => (x.assume() <= comparison).into(),
            MaybeImprecise::Imprecise(x) => match x {
                Comparison::Equal(x) => (*x <= comparison).into(),
                Comparison::GreaterThan(x) => (*x < comparison).into(),
                Comparison::GreaterThanOrEqual(x) => (*x < comparison).into(),
                Comparison::LowerThan(_) => Ternary::True,
                Comparison::LowerThanOrEqual(_) => Ternary::True,
                Comparison::NotEqual(_) => Ternary::True,
            },
        }
    }

    fn eq(&self, comparison: usize) -> Ternary {
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

    fn ne(&self, comparison: usize) -> Ternary {
        match self {
            MaybeImprecise::Precise(x) => (comparison != x.assume()).into(),
            MaybeImprecise::Imprecise(x) => match x {
                Comparison::Equal(x) => (comparison != *x).into(),
                Comparison::GreaterThan(_) => Ternary::True,
                Comparison::GreaterThanOrEqual(_) => Ternary::True,
                Comparison::LowerThan(_) => Ternary::True,
                Comparison::LowerThanOrEqual(_) => Ternary::True,
                Comparison::NotEqual(_) => Ternary::True,
            },
        }
    }
}

impl Compare for usize {
    fn gt(&self, comparison: usize) -> Ternary {
        (*self > comparison).into()
    }

    fn gt_eq(&self, comparison: usize) -> Ternary {
        (*self >= comparison).into()
    }

    fn lt(&self, comparison: usize) -> Ternary {
        (*self < comparison).into()
    }

    fn lt_eq(&self, comparison: usize) -> Ternary {
        (*self <= comparison).into()
    }

    fn eq(&self, comparison: usize) -> Ternary {
        (*self == comparison).into()
    }

    fn ne(&self, comparison: usize) -> Ternary {
        (*self != comparison).into()
    }
}
