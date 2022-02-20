#[derive(Debug)]
pub struct Card {
    suit: CardSuit,
    value: CardValue,
}

#[derive(Debug, Clone, Copy)]
pub enum CardSuit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}

impl CardSuit {
    pub fn all() -> [CardSuit; 4] {
        use CardSuit::*;
        [Spades, Hearts, Diamonds, Clubs]
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CardValue {
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

    pub fn health_value(&self) -> u16 {
        use CardValue::*;
        match self.value {
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
            Jack => 20,
            Queen => 30,
            King => 40,
        }
    }

    pub fn attack_value(&self) -> u16 {
        use CardValue::*;
        match self.value {
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
