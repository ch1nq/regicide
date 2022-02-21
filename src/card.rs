#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Card {
    pub suit: CardSuit,
    pub value: CardValue,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum CardSuit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
    None,
}

impl CardSuit {
    pub fn all() -> [CardSuit; 4] {
        use CardSuit::*;
        [Spades, Hearts, Diamonds, Clubs]
    }
}

impl PartialOrd for CardSuit {
    /// The ordering implemented is arbitrary and does not carry any meaning
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let suit_value = |suit| match suit {
            &CardSuit::Spades => Some(1),
            &CardSuit::Hearts => Some(2),
            &CardSuit::Diamonds => Some(3),
            &CardSuit::Clubs => Some(4),
            _ => None,
        };
        suit_value(self).partial_cmp(&suit_value(other))
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum CardValue {
    Jester,
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

impl CardValue {
    pub fn numbers() -> [CardValue; 10] {
        use CardValue::*;
        [Ace, Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten]
    }

    pub fn royals() -> [CardValue; 3] {
        use CardValue::*;
        [Jack, Queen, King]
    }
}

impl Card {
    pub fn new(suit: CardSuit, value: CardValue) -> Self {
        Self { suit, value }
    }

    pub fn attack_value(&self) -> u16 {
        use CardValue::*;
        match self.value {
            Jester => 0,
            Ace => 1,
            Two => 2,
            Three => 3,
            Four => 4,
            Five => 5,
            Six => 6,
            Seven => 7,
            Eight => 8,
            Nine => 9,
            Ten => 10,
            Jack => 10,
            Queen => 15,
            King => 20,
        }
    }
}
