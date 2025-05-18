use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Simple,
    Extended { saga: bool },
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Simple => write!(f, "Command"),
            Self::Extended { saga: false } => write!(f, "Extended Command"),
            Self::Extended { saga: true } => write!(f, "Extended Command Saga"),
        }
    }
}

impl Command {
    #[must_use]
    pub const fn get_name(self) -> &'static str {
        match self {
            Self::Simple => "command",
            Self::Extended { saga: false } => "extended command",
            Self::Extended { saga: true } => "extended command saga",
        }
    }
}

/// Represents a basic type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseType {
    Creature { saga: bool },
    Command(Command),
}

impl Display for BaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Creature { saga: false } => write!(f, "Creature"),
            Self::Creature { saga: true } => write!(f, "Creature Saga"),
            Self::Command(command) => write!(f, "{command}"),
        }
    }
}

impl BaseType {
    #[must_use]
    pub const fn get_name(&self) -> &'static str {
        match self {
            Self::Creature { saga: false } => "creature",
            Self::Creature { saga: true } => "creature saga",
            Self::Command(command) => command.get_name(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Represents a full card type
pub struct Type {
    base: BaseType,
    vestige: bool,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.base, self.vestige) {
            (BaseType::Creature { saga: true }, true) => write!(f, "Creature Saga Vestige"),
            (BaseType::Creature { saga: true }, false) => write!(f, "Creature Saga"),
            (BaseType::Creature { saga: false }, true) => write!(f, "Creature Vestige"),
            (BaseType::Creature { saga: false }, false) => write!(f, "Creature"),
            (BaseType::Command(Command::Simple), true) => write!(f, "Command Vestige"),
            (BaseType::Command(Command::Extended { saga: false }), true) => {
                write!(f, "Extended Command Vestige")
            }
            (BaseType::Command(Command::Extended { saga: true }), true) => {
                write!(f, "Extended Command Saga Vestige")
            }
            (BaseType::Command(Command::Simple), false) => write!(f, "Command"),
            (BaseType::Command(Command::Extended { saga: false }), false) => {
                write!(f, "Extended Command")
            }
            (BaseType::Command(Command::Extended { saga: true }), false) => {
                write!(f, "Extended Command Saga")
            }
        }
    }
}

impl Type {
    #[must_use]
    pub const fn get_name(&self) -> &'static str {
        match (self.base, self.vestige) {
            (BaseType::Creature { saga: true }, true) => "creature saga vestige",
            (BaseType::Creature { saga: true }, false) => "creature saga",
            (BaseType::Creature { saga: false }, true) => "creature vestige",
            (BaseType::Creature { saga: false }, false) => "creature",
            (BaseType::Command(Command::Simple), true) => "command vestige",
            (BaseType::Command(Command::Extended { saga: false }), true) => {
                "extended command vestige"
            }
            (BaseType::Command(Command::Extended { saga: true }), true) => {
                "extended command saga vestige"
            }
            (BaseType::Command(Command::Simple), false) => "command",
            (BaseType::Command(Command::Extended { saga: false }), false) => "extended command",
            (BaseType::Command(Command::Extended { saga: true }), false) => "extended command saga",
        }
    }

    #[must_use]
    pub fn from_string(string: &str) -> Option<Self> {
        match string {
            "creature saga vestige" => Some(Self {
                base: BaseType::Creature { saga: true },
                vestige: true,
            }),
            "creature saga" => Some(Self {
                base: BaseType::Creature { saga: true },
                vestige: false,
            }),
            "creature vestige" => Some(Self {
                base: BaseType::Creature { saga: false },
                vestige: true,
            }),
            "creature" => Some(Self {
                base: BaseType::Creature { saga: false },
                vestige: false,
            }),
            "command vestige" => Some(Self {
                base: BaseType::Command(Command::Simple),
                vestige: true,
            }),
            "extended command vestige" => Some(Self {
                base: BaseType::Command(Command::Extended { saga: false }),
                vestige: true,
            }),
            "extended command saga vestige" => Some(Self {
                base: BaseType::Command(Command::Extended { saga: true }),
                vestige: true,
            }),
            "command" => Some(Self {
                base: BaseType::Command(Command::Simple),
                vestige: false,
            }),
            "extended command" => Some(Self {
                base: BaseType::Command(Command::Extended { saga: false }),
                vestige: false,
            }),
            "extended command saga" => Some(Self {
                base: BaseType::Command(Command::Extended { saga: true }),
                vestige: false,
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Ternary bool
pub enum BoolPattern {
    True,
    False,
    Maybe,
}

impl BoolPattern {
    #[must_use]
    /// Operation on bools that returns true if they're both true, if they're both false, or if self is maybe
    pub const fn is_maybe_eq(self, other: bool) -> bool {
        match (self, other) {
            (Self::True, false) | (Self::False, true) => false,
            (Self::False | Self::Maybe, false) | (Self::True | Self::Maybe, true) => true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CommandPattern {
    Simple,
    Extended { saga: BoolPattern },
    MaybeExtended { saga: BoolPattern },
}

impl CommandPattern {
    #[must_use]
    pub const fn is_match(self, other: Command) -> bool {
        match (self, other) {
            (Self::Simple, Command::Extended { .. }) | (Self::Extended { .. }, Command::Simple) => {
                false
            }
            (
                Self::Extended { saga: self_saga } | Self::MaybeExtended { saga: self_saga },
                Command::Extended { saga: other_saga },
            ) => self_saga.is_maybe_eq(other_saga),
            (Self::MaybeExtended { .. } | Self::Simple, Command::Simple) => true,
        }
    }
}

/// Pattern to match against `BaseType`
#[derive(Debug, Clone, Copy)]
pub enum BaseTypePattern {
    Creature { saga: BoolPattern },
    Command(CommandPattern),
}

impl BaseTypePattern {
    #[must_use]
    pub const fn is_match(&self, other: BaseType) -> bool {
        match (self, other) {
            (Self::Creature { saga: self_saga }, BaseType::Creature { saga: other_saga }) => {
                self_saga.is_maybe_eq(other_saga)
            }
            (Self::Creature { .. }, BaseType::Command(_))
            | (Self::Command(_), BaseType::Creature { .. }) => false,
            (Self::Command(command_pat), BaseType::Command(command)) => {
                command_pat.is_match(command)
            }
        }
    }
}

/// Pattern to match against `Type`
#[derive(Debug, Clone, Copy)]
pub struct TypePattern {
    base: BaseTypePattern,
    vestige: BoolPattern,
}

impl TypePattern {
    #[must_use]
    pub const fn is_match(&self, other: Type) -> bool {
        self.base.is_match(other.base) && self.vestige.is_maybe_eq(other.vestige)
    }
}
