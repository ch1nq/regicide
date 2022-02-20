use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::VecDeque;
use std::error::Error;
use std::fmt::Formatter;

#[derive(Debug)]
struct Game {}
#[derive(Debug)]
pub enum Action {
    Play(Card),
    Yield,
}

#[derive(Debug)]
pub struct State {
    table: Table,
    players: Vec<Player>,
    has_turn: PlayerId,
    shield: u16,
}
#[derive(Debug)]
struct Table {
    castle_deck: Vec<Card>,
    tavern_deck: Vec<Card>,
    discard_pile: VecDeque<Card>,
}
#[derive(Debug)]
pub struct PlayerId(i8);
#[derive(Debug)]
struct Player {
    id: PlayerId,
    health: i16,
    hand: Vec<Card>,
}
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
    fn all() -> [CardSuit; 4] {
        use crate::logic::CardSuit::*;
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
    fn numbers() -> [CardValue; 10] {
        use crate::logic::CardValue::*;
        [Ace, Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten]
    }

    fn royals() -> [CardValue; 3] {
        use crate::logic::CardValue::*;
        [Jack, Queen, King]
    }
}

#[derive(Debug)]
pub enum GameResult {
    Won,
    Lost,
}
#[derive(Debug)]
pub enum GameStatus {
    InProgress(State),
    HasEnded(GameResult),
}

impl Card {
    fn health_value(&self) -> u16 {
        use crate::logic::CardValue::*;
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

    fn attack_value(&self) -> u16 {
        use crate::logic::CardValue::*;
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

impl Table {
    fn new(_n_jesters: u8) -> Self {
        let mut rng = thread_rng();

        let castle_deck = CardValue::royals()
            .iter()
            .flat_map(|value| {
                let mut level = CardSuit::all()
                    .iter()
                    .map(|suit| Card {
                        suit: *suit,
                        value: *value,
                    })
                    .collect::<Vec<Card>>();
                level.shuffle(&mut rng);
                level
            })
            .collect::<Vec<Card>>();

        // TODO: Add jesters into deck
        let mut tavern_deck = CardValue::numbers()
            .iter()
            .flat_map(|value| {
                CardSuit::all()
                    .iter()
                    .map(|suit| Card {
                        suit: suit.clone(),
                        value: value.clone(),
                    })
                    .collect::<Vec<Card>>()
            })
            .collect::<Vec<Card>>();

        tavern_deck.shuffle(&mut rng);

        // We can at most have all the cards in the deck in the discard pile
        let discard_pile = VecDeque::with_capacity(tavern_deck.len() + castle_deck.len());

        Self {
            castle_deck,
            tavern_deck,
            discard_pile,
        }
    }

    fn draw_cards(&mut self, n_cards: i8) -> Vec<Card> {
        (0..n_cards)
            .filter_map(|_| self.tavern_deck.pop())
            .collect()
    }
}

impl Player {
    fn new(id: i8, hand: Vec<Card>) -> Self {
        Self {
            id: PlayerId(id),
            health: Self::health_from_hand(&hand) as i16,
            hand,
        }
    }

    fn health_from_hand(hand: &Vec<Card>) -> u16 {
        hand.iter().map(|card| card.health_value()).sum()
    }
}

#[derive(Debug)]
pub enum StateError {
    WrongNumberOfPlayers,
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?})", self))
    }
}

impl Error for StateError {}

impl State {
    pub fn new(n_players: i8) -> Result<Self, StateError> {
        let (n_jesters, max_hand_size) = match n_players {
            1 => (0, 8),
            2 => (0, 7),
            3 => (1, 6),
            4 => (2, 5),
            _ => return Err(StateError::WrongNumberOfPlayers),
        };
        let mut table = Table::new(n_jesters);
        let players = (0..n_players)
            .map(|id| Player::new(id, table.draw_cards(max_hand_size)))
            .collect();

        Ok(Self {
            table,
            players,
            has_turn: PlayerId(0),
            shield: 0,
        })
    }

    pub fn take_action(&self, _action: &Action) -> GameStatus {
        todo!()
    }

    pub fn get_action_space(&self) -> Vec<Action> {
        todo!()
    }
}
