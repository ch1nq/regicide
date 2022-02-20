use crate::card::Card;

#[derive(Debug)]
pub struct PlayerId(pub i8);
#[derive(Debug)]
pub struct Player {
    id: PlayerId,
    health: i16,
    hand: Vec<Card>,
}

impl Player {
    pub fn new(id: i8, hand: Vec<Card>) -> Self {
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
