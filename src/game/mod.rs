pub mod card;
mod enemy;
mod player;
pub mod state;
mod table;

use std::fmt::Debug;

use card::Card;
use state::State;

use self::player::PlayerId;

#[derive(Debug, Clone, Copy, Hash)]
pub enum GameResult {
    Won,
    Lost(u8),
}
#[derive(Debug)]
pub enum GameStatus {
    InProgress(State),
    HasEnded(GameResult),
}

#[derive(Debug)]
struct Game {}
#[derive(Debug, Clone)]
pub enum Action {
    Play(Card),
    AnimalCombo(Card, Card),
    Combo2(Card, Card),
    Combo3(Card, Card, Card),
    Combo4(Card, Card, Card, Card),
    Yield,
    Discard(Vec<Card>),
    ChangePlayer(PlayerId),
}
