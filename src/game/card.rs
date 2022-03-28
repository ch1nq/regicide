use core::fmt;

use arrayvec::ArrayVecCopy;

pub type CardVec = ArrayVecCopy<Card, 54>;
pub type Hand = ArrayVecCopy<Card, { super::MAX_HAND_SIZE }>;

pub trait FromCardIter {
    fn from_card_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Card>;
}

impl<const N: usize> FromCardIter for ArrayVecCopy<Card, N> {
    fn from_card_iter<T>(iter: T) -> ArrayVecCopy<Card, N>
    where
        T: IntoIterator<Item = Card>,
    {
        let mut arr = ArrayVecCopy::<Card, N>::new();
        arr.extend(iter.into_iter());
        arr
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Card {
    pub suit: CardSuit,
    pub value: CardValue,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
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

pub trait AttackValue: Sized + Copy {
    fn attack_value(&self) -> u16;
}

/// Shorthand for getting the sum of all attack values from a vec
pub trait AttackSum {
    fn attack_sum(&self) -> u16;
}

impl<T> AttackSum for Vec<T>
where
    T: AttackValue,
{
    fn attack_sum(&self) -> u16 {
        self.iter().map(|card| card.attack_value()).sum()
    }
}

impl AttackSum for Hand {
    fn attack_sum(&self) -> u16 {
        self.iter().map(|card| card.attack_value()).sum()
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

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
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
        [King, Queen, Jack]
    }
}

impl Card {
    pub fn new(suit: CardSuit, value: CardValue) -> Self {
        Self { suit, value }
    }
}

impl AttackValue for Card {
    fn attack_value(&self) -> u16 {
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

impl<'a> AttackValue for &'a Card {
    fn attack_value(&self) -> u16 {
        (*self).attack_value()
    }
}

impl fmt::Debug for CardSuit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CardSuit::*;
        let suit = match self {
            Spades => "♠",
            Hearts => "♥",
            Diamonds => "♦",
            Clubs => "♣",
            _ => "",
        };

        write!(f, "{}", suit)
    }
}

impl fmt::Debug for CardValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CardValue::*;
        let value = match self {
            Jester => "Jester",
            Ace => "A",
            Two => "2",
            Three => "3",
            Four => "4",
            Five => "5",
            Six => "6",
            Seven => "7",
            Eight => "8",
            Nine => "9",
            Ten => "10",
            Jack => "J",
            Queen => "Q",
            King => "K",
        };

        write!(f, "{}", value)
    }
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}{:?}", self.suit, self.value)
    }
}
