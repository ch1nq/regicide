use crate::card::Card;
use crate::state::State;

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
#[derive(Debug)]
pub enum Action {
    Play(Card),
    AnimalCombo(Card, Card),
    Combo2(Card, Card),
    Combo3(Card, Card, Card),
    Combo4(Card, Card, Card, Card),
    Yield,
}
