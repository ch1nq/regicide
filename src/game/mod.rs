pub mod card;
pub mod enemy;
pub mod player;
pub mod policy;
pub mod state;
pub mod table;

use self::{card::Hand, player::PlayerId};
use card::Card;
use state::State;
use std::fmt::Debug;

pub const MAX_HAND_SIZE: usize = 8;

#[derive(Debug, Clone, Copy, Hash)]
pub enum GameResult {
    Won,
    Lost(u8),
}
#[derive(Debug)]
pub enum GameStatus<const N_PLAYERS: usize> {
    InProgress(State<N_PLAYERS>),
    HasEnded(GameResult),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Play(Card),
    AnimalCombo(Card, Card),
    Combo(arrayvec::ArrayVecCopy<Card, 4>),
    Yield,
    Discard(Hand),
    ChangePlayer(PlayerId),
    RefillHand,
}

impl GameResult {
    pub const fn max_score() -> u8 {
        12
    }
}
