use super::{Comparison, ImpreciseEq, MaybeImprecise, MaybeVar};

impl ImpreciseEq<Self> for Comparison {
    fn imprecise_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Equal(x),
                Self::Equal(y) | Self::GreaterThanOrEqual(y) | Self::LowerThanOrEqual(y),
            ) => x == y,
            (Self::GreaterThan(x), Self::GreaterThan(y) | Self::Equal(y) | Self::NotEqual(y)) => {
                x > y
            }
            (
                Self::GreaterThanOrEqual(x),
                Self::GreaterThanOrEqual(y)
                | Self::LowerThanOrEqual(y)
                | Self::GreaterThan(y)
                | Self::Equal(y),
            ) => x >= y,
            (Self::LowerThan(x), Self::LowerThan(y) | Self::Equal(y) | Self::NotEqual(y)) => x < y,
            (
                Self::LowerThanOrEqual(x),
                Self::LowerThanOrEqual(y)
                | Self::GreaterThanOrEqual(y)
                | Self::LowerThan(y)
                | Self::Equal(y),
            ) => x <= y,
            (Self::NotEqual(_), Self::Equal(_)) => false,
            (Self::NotEqual(_), _) => true,
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

impl ImpreciseEq<Self> for MaybeVar {
    fn imprecise_eq(&self, other: &Self) -> bool {
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

impl ImpreciseEq<Self> for MaybeImprecise {
    fn imprecise_eq(&self, other: &Self) -> bool {
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
