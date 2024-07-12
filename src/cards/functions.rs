use crate::numbers::MaybeImprecise;

type Identity = Vec<Restriction>;

pub struct Rule {
    bindings: Vec<Identity>,
    actions: Vec<Action>,
}

pub enum Action {
    Nothing,
    Shuffle(Deck),
    Draw(Deck),
    Search(usize),
    Destroy(usize),
    Sacrifice(usize),
    Sequence(Vec<Action>),
    Or(Vec<Action>),
    Payment {
        cost: Box<Action>,
        reward: Box<Action>,
    },
    Xor(Vec<Action>),
    LoseBlood,
    Repeat(Box<Action>, usize),
    Must(Box<Action>),
    CreateVestige {
        vestige: usize,
        zone: Zone,
    },
}

pub enum Zone {
    Deck(Deck),
    Hand,
    Board,
}

pub enum Deck {
    Blood,
    Main,
}

pub enum Type {
    Creature,
    Command,
    Vestige(Box<Type>),
}

pub enum Restriction {
    Type(Type),
    Cost(MaybeImprecise),
    Name(String),
}

#[allow(clippy::no_effect)]
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn function() {
        // Green Queen
        Rule {
            bindings: vec![],
            actions: vec![Action::Or(vec![Action::Payment {
                cost: Box::new(Action::Repeat(Box::new(Action::LoseBlood), 2)),
                reward: Box::new(Action::Repeat(Box::new(Action::Draw(Deck::Main)), 2)),
            }])],
        };

        // Deranged Researcher
        Rule {
            bindings: vec![vec![Restriction::Name("Grand Design".to_string())]],
            actions: vec![Action::Sequence(vec![
                Action::CreateVestige {
                    vestige: 0,
                    zone: Zone::Deck(Deck::Main),
                },
                Action::Shuffle(Deck::Main),
            ])],
        };
    }
}
