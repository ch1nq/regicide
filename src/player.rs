use std::convert::TryInto;

use crate::card::Card;

#[derive(Debug, Clone)]
pub struct PlayerId(pub usize);

impl PlayerId {
    pub fn next_id(&self, total_players: usize) -> Self {
        PlayerId((self.0 + 1) % (total_players - 1))
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    id: PlayerId,
    pub hand: Vec<Card>,
}

impl Player {
    pub fn new(id: usize, hand: Vec<Card>) -> Self {
        Self {
            id: PlayerId(id),
            hand,
        }
    }

    pub fn total_hand_value(&self) -> u16 {
        self.hand.iter().map(|card| card.attack_value()).sum()
    }
}
