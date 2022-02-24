pub mod card;
mod enemy;
mod player;
pub mod state;
mod table;

use card::Card;
use state::State;

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
}
