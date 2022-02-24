use crate::card::{Card, CardSuit, CardValue};
use crate::enemy::Enemy;
use crate::error::RegicideError;
use rand::prelude::{SliceRandom, ThreadRng};
use rand::thread_rng;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct Table {
    castle_deck: Vec<Enemy>,
    tavern_deck: VecDeque<Card>,
    discard_pile: Vec<Card>,
    attack_cards: Vec<Card>,
}

impl Table {
    pub fn new(n_jesters: u8) -> Result<Self, RegicideError> {
        let mut rng = thread_rng();

        let castle_deck = Self::castle_deck(&mut rng)?;
        let tavern_deck = Self::tavern_deck(&mut rng, n_jesters);
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

    fn castle_deck(rng: &mut ThreadRng) -> Result<Vec<Enemy>, RegicideError> {
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

    fn tavern_deck(rng: &mut ThreadRng, n_jesters: u8) -> VecDeque<Card> {
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

    pub fn heal_from_discard(&mut self, n_cards: u8) {
        self.discard_pile.shuffle(&mut thread_rng());
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
}
