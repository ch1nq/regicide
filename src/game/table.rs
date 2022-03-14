use super::card::{Card, CardSuit, CardValue};
use super::enemy::Enemy;
use crate::error::RegicideError;
use itertools::Itertools;
use rand::prelude::{SliceRandom, ThreadRng};
use rand::rngs::StdRng;
use std::collections::VecDeque;

#[derive(Debug, Clone, Hash)]
pub struct Table {
    castle_deck: Vec<Enemy>,
    tavern_deck: VecDeque<Card>,
    discard_pile: Vec<Card>,
    attack_cards: Vec<Card>,
}

impl Table {
    pub fn new(n_jesters: u8, rng: &mut StdRng) -> Result<Self, RegicideError> {
        let castle_deck = Self::new_castle_deck(rng)?;
        let tavern_deck = Self::new_tavern_deck(rng, n_jesters);
        let attack_cards = vec![];

        // We can at most have all the cards in the deck in the discard pile
        let discard_pile = Vec::with_capacity(tavern_deck.len() + castle_deck.len());

        Ok(Table {
            castle_deck,
            tavern_deck,
            discard_pile,
            attack_cards,
        })
    }

    fn new_castle_deck(rng: &mut StdRng) -> Result<Vec<Enemy>, RegicideError> {
        CardValue::royals()
            .iter()
            .flat_map(|value| {
                let mut level = CardSuit::all()
                    .iter()
                    .map(|suit| Card::new(*suit, *value))
                    .map(|card| Enemy::new(card))
                    .collect::<Vec<Result<Enemy, RegicideError>>>();
                level.shuffle(rng);
                level
            })
            .collect()
    }

    fn new_tavern_deck(rng: &mut StdRng, n_jesters: u8) -> VecDeque<Card> {
        let mut tavern_deck = CardValue::numbers()
            .iter()
            .flat_map(|value| {
                CardSuit::all()
                    .iter()
                    .map(|suit| Card::new(*suit, *value))
                    .collect::<Vec<Card>>()
            })
            .collect::<Vec<Card>>();

        // Add jesters into deck
        tavern_deck.append(&mut vec![
            Card::new(CardSuit::None, CardValue::Jester);
            n_jesters.into()
        ]);

        tavern_deck.shuffle(rng);
        VecDeque::from(tavern_deck)
    }

    pub fn draw_cards(&mut self, n_cards: u8) -> Vec<Card> {
        (0..n_cards)
            .filter_map(|_| self.tavern_deck.pop_front())
            .collect()
    }

    pub fn discard_card(&mut self, card: Card) {
        self.discard_pile.push(card);
    }

    pub fn add_attack_cards(&mut self, cards: &Vec<Card>) {
        self.attack_cards.extend(cards.iter());
    }

    pub fn attack_cards(&self) -> &Vec<Card> {
        &self.attack_cards
    }

    /// Place all cards played by players against the enemy in the discard pile.
    pub fn discard_attack_cards(&mut self) {
        self.discard_pile.append(&mut self.attack_cards);
    }

    pub fn add_to_top_of_tavern_deck(&mut self, card: Card) {
        self.tavern_deck.push_front(card);
    }

    pub fn heal_from_discard(&mut self, n_cards: u8, rng: &mut StdRng) {
        self.discard_pile.shuffle(rng);
        let mut cards = (0..n_cards)
            .filter_map(|_| self.discard_pile.pop())
            .collect::<VecDeque<Card>>();
        self.tavern_deck.append(&mut cards);
    }

    pub fn current_enemy(&self) -> Option<&Enemy> {
        self.castle_deck.last()
    }

    pub fn current_enemy_mut(&mut self) -> Option<&mut Enemy> {
        self.castle_deck.last_mut()
    }

    /// Turn the next card of the Castle deck face up.
    pub fn next_enemy(&mut self) {
        self.castle_deck.pop();
    }

    pub fn permute(&mut self, player_hands: Vec<&mut Vec<Card>>, rng: &mut StdRng) {
        // Shuffle castle deck
        let castle_deck = Self::new_castle_deck(rng)
            .unwrap()
            .iter()
            .filter(|enemy| self.castle_deck.contains(enemy))
            .map(|c| *c)
            .collect_vec();
        self.castle_deck = castle_deck;

        // Combine all potentially unknown cards into a single pile:
        // hands + discard pile + tavern deck
        let discard_pile = self.discard_pile.clone();
        let tavern_deck = self.tavern_deck.iter().map(|c| *c).collect_vec();
        let player_cards = player_hands
            .iter()
            .flat_map(|hand| hand.iter())
            .map(|c| *c)
            .collect_vec();
        let mut combined_cards = [discard_pile, player_cards, tavern_deck].concat();
        combined_cards.shuffle(rng);

        // Distribute cards back out
        for hand in player_hands {
            *hand = combined_cards.drain(..hand.len()).collect();
        }
        self.tavern_deck =
            VecDeque::from(combined_cards.drain(..self.tavern_deck.len()).collect_vec());

        self.discard_pile = combined_cards;
    }
}
