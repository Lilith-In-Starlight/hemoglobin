use std::cmp::Ordering;

use super::{Comparison, ImpreciseOrd, MaybeImprecise, MaybeVar};

impl<T: ImpreciseOrd<T>> ImpreciseOrd<Self> for Option<T> {
    fn imprecise_cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Some(x), Some(y)) => x.imprecise_cmp(y),
            (Some(_), None) => Ordering::Greater,
            (None, Some(_) | None) => Ordering::Less,
        }
    }
}

impl<U, T: ImpreciseOrd<U>> ImpreciseOrd<&U> for &T {
    fn imprecise_cmp(&self, other: &&U) -> Ordering {
        (*self).imprecise_cmp(*other)
    }
}

impl ImpreciseOrd<Self> for Comparison {
    fn imprecise_cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::GreaterThan(x), Self::GreaterThan(y) | Self::Equal(y) | Self::NotEqual(y)) => {
                x.cmp(y)
            }
            (
                Self::GreaterThanOrEqual(x),
                Self::GreaterThanOrEqual(y)
                | Self::LowerThanOrEqual(y)
                | Self::GreaterThan(y)
                | Self::Equal(y),
            )
            | (
                Self::Equal(x),
                Self::Equal(y) | Self::GreaterThanOrEqual(y) | Self::LowerThanOrEqual(y),
            ) => x.cmp(y),
            (Self::LowerThan(x), Self::LowerThan(y) | Self::Equal(y) | Self::NotEqual(y)) => {
                x.cmp(y).reverse()
            }
            (
                Self::LowerThanOrEqual(x),
                Self::LowerThanOrEqual(y)
                | Self::GreaterThanOrEqual(y)
                | Self::LowerThan(y)
                | Self::Equal(y),
            ) => x.cmp(y).reverse(),
            (Self::NotEqual(_), Self::Equal(_)) => Ordering::Less,
            (Self::NotEqual(_), _) => Ordering::Equal,
            _ => Ordering::Less,
        }
    }
}

// impl ImpreciseOrd<usize> for Comparison {
//     fn imprecise_cmp(&self, other: &usize) -> Ordering {
//         match self {
//             Self::GreaterThan(x) => {
//                 if other >= x {
//                     x.cmp(other)
//                 } else {
//                     Ordering::Less
//                 }
//             }
//         }
//     }
// }

// // Reverse
// impl ImpreciseOrd<Comparison> for usize {
//     fn imprecise_cmp(&self, other: &Comparison) -> Ordering {
//         other.imprecise_cmp(self)
//     }
// }

impl ImpreciseOrd<Self> for MaybeVar {
    fn imprecise_cmp(&self, other: &Self) -> Ordering {
        self.assume().cmp(&other.assume())
    }
}

// impl ImpreciseOrd<usize> for MaybeVar {
//     fn imprecise_cmp(&self, other: &usize) -> Ordering {
//         self.assume() == *other
//     }
// }

// // Reverse
// impl ImpreciseOrd<MaybeVar> for usize {
//     fn imprecise_cmp(&self, other: &MaybeVar) -> Ordering {
//         other.imprecise_cmp(self)
//     }
// }

impl ImpreciseOrd<MaybeVar> for Comparison {
    fn imprecise_cmp(&self, other: &MaybeVar) -> Ordering {
        self.imprecise_cmp(&Self::Equal(other.assume()))
    }
}

// Reverse
impl ImpreciseOrd<Comparison> for MaybeVar {
    fn imprecise_cmp(&self, other: &Comparison) -> Ordering {
        other.imprecise_cmp(self).reverse()
    }
}

impl ImpreciseOrd<Self> for MaybeImprecise {
    fn imprecise_cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Precise(x), Self::Imprecise(y)) | (Self::Imprecise(y), Self::Precise(x)) => {
                x.imprecise_cmp(y)
            }
            (Self::Precise(x), Self::Precise(y)) => x.imprecise_cmp(y),
            (Self::Imprecise(x), Self::Imprecise(y)) => x.imprecise_cmp(y),
        }
    }
}

impl ImpreciseOrd<Comparison> for MaybeImprecise {
    fn imprecise_cmp(&self, other: &Comparison) -> Ordering {
        match self {
            Self::Precise(x) => x.imprecise_cmp(other),
            Self::Imprecise(x) => x.imprecise_cmp(other),
        }
    }
}

// Reverse
impl ImpreciseOrd<MaybeImprecise> for Comparison {
    fn imprecise_cmp(&self, other: &MaybeImprecise) -> Ordering {
        other.imprecise_cmp(self).reverse()
    }
}

// impl ImpreciseOrd<usize> for MaybeImprecise {
//     fn imprecise_cmp(&self, other: &usize) -> Ordering {
//         match self {
//             Self::Precise(x) => x.imprecise_cmp(other),
//             Self::Imprecise(x) => x.imprecise_cmp(other),
//         }
//     }
// }

// // Reverse
// impl ImpreciseOrd<MaybeImprecise> for usize {
//     fn imprecise_cmp(&self, other: &MaybeImprecise) -> Ordering {
//         other.imprecise_cmp(self)
//     }
// }
// impl ImpreciseOrd<MaybeVar> for MaybeImprecise {
//     fn imprecise_cmp(&self, other: &MaybeVar) -> Ordering {
//         match self {
//             Self::Precise(x) => x.imprecise_cmp(other),
//             Self::Imprecise(x) => x.imprecise_cmp(other),
//         }
//     }
// }

// // Reverse
// impl ImpreciseOrd<MaybeImprecise> for MaybeVar {
//     fn imprecise_cmp(&self, other: &MaybeImprecise) -> Ordering {
//         other.imprecise_cmp(self)
//     }
// }
