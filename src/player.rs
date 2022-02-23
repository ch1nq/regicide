use std::slice::SliceIndex;

use crate::card::Card;

#[derive(Debug, Clone)]
pub struct PlayerId(pub i8);

#[derive(Debug, Clone)]
pub struct Player {
    id: PlayerId,
    health: i16,
    pub hand: Vec<Card>,
}

impl Player {
    pub fn new(id: i8, hand: Vec<Card>) -> Self {
        Self {
            id: PlayerId(id),
            health: Self::total_hand_value(&hand) as i16,
            hand,
        }
    }

    pub fn total_hand_value(hand: &Vec<Card>) -> u16 {
        hand.iter().map(|card| card.attack_value()).sum()
    }
}
