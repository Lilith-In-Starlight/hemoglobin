use chumsky::container::Seq;
use regex::Regex;
use serde::{de::Visitor, Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Represents a Kin. To represent the Kin Tree, some variants have an Option. The None variant represents the parent Kin, while the Some variant represents a child.
pub enum Kin {
    Sorcery,
    Their,
    Insect(Option<InsectKin>),
    Piezan(Option<PiezanKin>),
    Machine(Option<MachineKin>),
}

impl Kin {
    pub fn get_name(self) -> &'static str {
        match self {
            Self::Sorcery => "sorcery",
            Self::Insect(insect_kin) => insect_kin.map_or("insect", InsectKin::get_name),
            Self::Piezan(piezan_kin) => piezan_kin.map_or("piezan", PiezanKin::get_name),
            Self::Machine(machine_kin) => machine_kin.map_or("machine", MachineKin::get_name),
            Self::Their => "THEIR",
        }
    }

    /// Returns true if `other` is the same kin or a child kin of `self`
    pub fn is_same_or_child(self, other: Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            // Childless kins
            (Self::Sorcery, Self::Sorcery) => true,
            (Self::Their, Self::Their) => true,
            (_, Self::Sorcery) | (Self::Sorcery, _) => false,
            (_, Self::Their) | (Self::Their, _) => false,
            // Insect kins
            (Self::Insect(_), Self::Insect(None)) => true,
            (Self::Insect(None), Self::Insect(_)) => false,
            (Self::Insect(Some(a)), Self::Insect(Some(b))) => a == b,
            // Piezan kins
            (Self::Piezan(_), Self::Piezan(None)) => true,
            (Self::Piezan(None), Self::Piezan(_)) => false,
            (Self::Piezan(Some(a)), Self::Piezan(Some(b))) => a == b,
            // Machine kins
            (Self::Machine(_), Self::Machine(None)) => true,
            (Self::Machine(None), Self::Machine(_)) => false,
            (Self::Machine(Some(a)), Self::Machine(Some(b))) => a == b,
            _ => false,
        }
    }

    pub fn get_equalness(self, other: Self) -> f64 {
        if self.is_same_or_child(other) {
            if self == other {
                1.0
            } else {
                0.5
            }
        } else {
            0.0
        }
    }

    pub fn from_string<'a>(string: &'a str) -> Option<Self> {
        match string {
            "THEIR" | "their" | "THEY" | "they" => Some(Kin::Their),
            "sorcery" => Some(Kin::Sorcery),
            "insect" => Some(Kin::Insect(None)),
            "piezan" => Some(Kin::Piezan(None)),
            "machine" => Some(Kin::Machine(None)),
            "ant" => Some(Kin::Insect(Some(InsectKin::Ant))),
            "bee" => Some(Kin::Insect(Some(InsectKin::Bee))),
            "blight" => Some(Kin::Machine(Some(MachineKin::Blight))),
            "red kingdom" => Some(Kin::Piezan(Some(PiezanKin::RedKingdom))),
            "blue kingdom" => Some(Kin::Piezan(Some(PiezanKin::BlueKingdom))),
            "green kingdom" => Some(Kin::Piezan(Some(PiezanKin::GreenKingdom))),
            "black kingdom" => Some(Kin::Piezan(Some(PiezanKin::BlackKingdom))),
            _ => None,
        }
    }
}

impl Serialize for Kin {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.get_name())
    }
}

struct KinVisitor;
impl Visitor<'_> for KinVisitor {
    type Value = Kin;

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Self::Value::from_string(v).ok_or(E::custom(format!("{v} is not a valid kin")))
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a valid kin name")
    }
}

impl<'de> Deserialize<'de> for Kin {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(KinVisitor)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsectKin {
    Ant,
    Bee,
}

impl InsectKin {
    #[must_use]
    pub const fn get_name(self) -> &'static str {
        match self {
            Self::Ant => "ant",
            Self::Bee => "bee",
        }
    }

    pub fn get_equalness(self, other: Self) -> f64 {
        if self == other {
            1.0
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PiezanKin {
    RedKingdom,
    BlueKingdom,
    BlackKingdom,
    GreenKingdom,
}

impl PiezanKin {
    #[must_use]
    pub const fn get_name(self) -> &'static str {
        match self {
            Self::RedKingdom => "red kingdom",
            Self::BlueKingdom => "blue kingdom",
            Self::BlackKingdom => "black kingdom",
            Self::GreenKingdom => "green kingdom",
        }
    }
    pub fn get_equalness(self, other: Self) -> f64 {
        if self == other {
            1.0
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineKin {
    Blight,
}

impl MachineKin {
    #[must_use]
    pub const fn get_name(self) -> &'static str {
        match self {
            Self::Blight => "blight",
        }
    }
    pub fn get_equalness(self, other: Self) -> f64 {
        if self == other {
            1.0
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deser() {
        println!(
            "{}",
            serde_json::to_string_pretty(&Kin::Insect(Some(InsectKin::Ant))).unwrap()
        );
        panic!()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Represents a comparison between Kins.
pub enum KinComparison {
    Equal(Kin),
    Similar(Kin),
    TextContains(String),
    TextEqual(String),
    #[serde(with = "serde_regex")]
    RegexMatch(Regex),
}

impl KinComparison {
    /// `Self::Equal` will match only exactly the same kin.
    /// `Self::Similar` will match the same kin as well as child kins (see: `Kin::is_same_or_child`)
    #[must_use]
    pub fn is_match(self, other: Kin) -> bool {
        match self {
            Self::Equal(this) => this == other,
            Self::Similar(this) => this.is_same_or_child(other),
            Self::TextContains(this) => other.get_name().contains(&this),
            Self::TextEqual(this) => other.get_name().to_lowercase() == this.to_lowercase(),
            Self::RegexMatch(regex) => regex.is_match(&other.get_name()),
        }
    }
}
