use crate::game::card::{AttackSum, Hand};
use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub struct PlayerId(pub usize);

impl PlayerId {
    pub fn next_id(&self, total_players: usize) -> Self {
        PlayerId((self.0 + 1) % total_players)
    }
}

#[derive(Debug, Clone, Copy, Hash)]
#[pyclass]
pub struct Player {
    id: PlayerId,
    pub hand: Hand,
}

impl Player {
    pub fn new(id: usize, hand: Hand) -> Self {
        Self {
            id: PlayerId(id),
            hand,
        }
    }

    pub fn id(&self) -> PlayerId {
        self.id
    }

    pub fn total_hand_value(&self) -> u16 {
        self.hand.attack_sum()
    }

    /// Removes specified cards from the players hand if they are present.
    pub fn remove_from_hand(&mut self, cards: &Hand) {
        self.hand = self
            .hand
            .iter()
            .filter(|card| !cards.contains(card))
            .copied()
            .collect();
    }
}
