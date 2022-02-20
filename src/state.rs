use crate::game::{Action, GameStatus};
use crate::player::{Player, PlayerId};
use crate::table::Table;
use std::error::Error;
use std::fmt::Formatter;

#[derive(Debug)]
pub struct State {
    table: Table,
    players: Vec<Player>,
    has_turn: PlayerId,
    shield: u16,
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
