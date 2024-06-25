use super::{Comparison, ImpreciseEq, MaybeImprecise, MaybeVar};

impl ImpreciseEq<Self> for Comparison {
    fn imprecise_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Comparison::Equal(x),
                Comparison::Equal(y)
                | Comparison::GreaterThanOrEqual(y)
                | Comparison::LowerThanOrEqual(y),
            ) => x == y,
            (
                Comparison::GreaterThan(x),
                Comparison::GreaterThan(y) | Comparison::Equal(y) | Comparison::NotEqual(y),
            ) => x > y,
            (
                Comparison::GreaterThanOrEqual(x),
                Comparison::GreaterThanOrEqual(y)
                | Comparison::LowerThanOrEqual(y)
                | Comparison::GreaterThan(y)
                | Comparison::Equal(y),
            ) => x >= y,
            (
                Comparison::LowerThan(x),
                Comparison::LowerThan(y) | Comparison::Equal(y) | Comparison::NotEqual(y),
            ) => x < y,
            (
                Comparison::LowerThanOrEqual(x),
                Comparison::LowerThanOrEqual(y)
                | Comparison::GreaterThanOrEqual(y)
                | Comparison::LowerThan(y)
                | Comparison::Equal(y),
            ) => x <= y,
            (Comparison::NotEqual(_), Comparison::Equal(_)) => false,
            (Comparison::NotEqual(_), _) => true,
            _ => false,
        }
    }
}

impl ImpreciseEq<usize> for Comparison {
    fn imprecise_eq(&self, other: &usize) -> bool {
        self.compare(other).into()
    }
}

// Reverse
impl ImpreciseEq<Comparison> for usize {
    fn imprecise_eq(&self, other: &Comparison) -> bool {
        other.imprecise_eq(self)
    }
}

impl ImpreciseEq<MaybeVar> for MaybeVar {
    fn imprecise_eq(&self, other: &MaybeVar) -> bool {
        self.assume() == other.assume()
    }
}

impl ImpreciseEq<usize> for MaybeVar {
    fn imprecise_eq(&self, other: &usize) -> bool {
        self.assume() == *other
    }
}

// Reverse
impl ImpreciseEq<MaybeVar> for usize {
    fn imprecise_eq(&self, other: &MaybeVar) -> bool {
        other.imprecise_eq(self)
    }
}

impl ImpreciseEq<MaybeVar> for Comparison {
    fn imprecise_eq(&self, other: &MaybeVar) -> bool {
        self.compare(&other.assume()).into()
    }
}

// Reverse
impl ImpreciseEq<Comparison> for MaybeVar {
    fn imprecise_eq(&self, other: &Comparison) -> bool {
        other.imprecise_eq(self)
    }
}

impl ImpreciseEq<MaybeImprecise> for MaybeImprecise {
    fn imprecise_eq(&self, other: &MaybeImprecise) -> bool {
        match (self, other) {
            (Self::Precise(x), Self::Imprecise(y)) | (Self::Imprecise(y), Self::Precise(x)) => {
                x.imprecise_eq(y)
            }
            (Self::Precise(x), Self::Precise(y)) => x.imprecise_eq(y),
            (Self::Imprecise(x), Self::Imprecise(y)) => x.imprecise_eq(y),
        }
    }
}

impl ImpreciseEq<usize> for MaybeImprecise {
    fn imprecise_eq(&self, other: &usize) -> bool {
        match self {
            Self::Precise(x) => x.imprecise_eq(other),
            Self::Imprecise(x) => x.imprecise_eq(other),
        }
    }
}

// Reverse
impl ImpreciseEq<MaybeImprecise> for usize {
    fn imprecise_eq(&self, other: &MaybeImprecise) -> bool {
        other.imprecise_eq(self)
    }
}
impl ImpreciseEq<MaybeVar> for MaybeImprecise {
    fn imprecise_eq(&self, other: &MaybeVar) -> bool {
        match self {
            Self::Precise(x) => x.imprecise_eq(other),
            Self::Imprecise(x) => x.imprecise_eq(other),
        }
    }
}

// Reverse
impl ImpreciseEq<MaybeImprecise> for MaybeVar {
    fn imprecise_eq(&self, other: &MaybeImprecise) -> bool {
        other.imprecise_eq(self)
    }
}
