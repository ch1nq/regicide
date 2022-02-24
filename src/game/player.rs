use super::card::{AttackSum, Card};

#[derive(Debug, Clone, Copy)]
pub struct PlayerId(pub usize);

impl PlayerId {
    pub fn next_id(&self, total_players: usize) -> Self {
        PlayerId((self.0 + 1) % total_players)
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

    pub fn id(&self) -> PlayerId {
        self.id
    }

    pub fn total_hand_value(&self) -> u16 {
        self.hand.attack_sum()
    }

    /// Removes specified cards from the players hand if they are present.
    pub fn remove_from_hand(&mut self, cards: &Vec<Card>) {
        self.hand = self
            .hand
            .iter()
            .filter(|card| !cards.contains(card))
            .map(|c| *c)
            .collect();
    }
}
