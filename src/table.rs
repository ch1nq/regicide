use crate::card::{Card, CardSuit, CardValue};
use rand::prelude::SliceRandom;
use rand::thread_rng;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Table {
    castle_deck: Vec<Card>,
    tavern_deck: Vec<Card>,
    discard_pile: VecDeque<Card>,
}

impl Table {
    pub fn new(_n_jesters: u8) -> Self {
        let mut rng = thread_rng();

        let castle_deck = CardValue::royals()
            .iter()
            .flat_map(|value| {
                let mut level = CardSuit::all()
                    .iter()
                    .map(|suit| Card::new(*suit, *value))
                    .collect::<Vec<Card>>();
                level.shuffle(&mut rng);
                level
            })
            .collect::<Vec<Card>>();

        // TODO: Add jesters into deck
        let mut tavern_deck = CardValue::numbers()
            .iter()
            .flat_map(|value| {
                CardSuit::all()
                    .iter()
                    .map(|suit| Card::new(*suit, *value))
                    .collect::<Vec<Card>>()
            })
            .collect::<Vec<Card>>();

        tavern_deck.shuffle(&mut rng);

        // We can at most have all the cards in the deck in the discard pile
        let discard_pile = VecDeque::with_capacity(tavern_deck.len() + castle_deck.len());

        Self {
            castle_deck,
            tavern_deck,
            discard_pile,
        }
    }

    pub fn draw_cards(&mut self, n_cards: i8) -> Vec<Card> {
        (0..n_cards)
            .filter_map(|_| self.tavern_deck.pop())
            .collect()
    }
}
