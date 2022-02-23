use crate::card::{Card, CardSuit, CardValue};
use rand::prelude::SliceRandom;
use rand::thread_rng;
use std::collections::VecDeque;
use std::convert::TryInto;

#[derive(Debug, Clone)]
pub struct Table {
    castle_deck: Vec<Enemy>,
    tavern_deck: VecDeque<Card>,
    discard_pile: Vec<Card>,
    attack_cards: Vec<Card>,
}

#[derive(Debug, Clone)]
pub struct Enemy {
    card: Card,
    health: i8,
    attack: u8,
}

#[derive(Debug)]
pub enum EnemyError {
    NotAnEnemy(Card),
}

impl Enemy {
    fn new(card: Card) -> Result<Enemy, EnemyError> {
        use CardValue::*;
        let health = match card.value {
            Jack => 20,
            Queen => 30,
            King => 40,
            _ => return Err(EnemyError::NotAnEnemy(card)),
        };
        Ok(Self {
            card,
            health,
            attack: card.attack_value() as u8,
        })
    }

    pub fn health(&self) -> i8 {
        self.health
    }

    pub fn card(&self) -> &Card {
        &self.card
    }

    pub fn take_damage(&mut self, amount: u16) {
        self.health = self
            .health
            .checked_sub(amount.try_into().expect("That's a lot of damage!"))
            .unwrap_or(-1);
    }

    pub fn decrease_attack(&mut self, by: u16) {
        self.attack = self
            .attack
            .checked_sub(by.try_into().expect("That's a lot of damage!"))
            .unwrap_or(0);
    }
}

impl Table {
    pub fn new(n_jesters: u8) -> Self {
        let mut rng = thread_rng();

        let castle_deck = CardValue::royals()
            .iter()
            .flat_map(|value| {
                let mut level = CardSuit::all()
                    .iter()
                    .map(|suit| Card::new(*suit, *value))
                    .map(|card| Enemy::new(card).unwrap())
                    .collect::<Vec<Enemy>>();
                level.shuffle(&mut rng);
                level
            })
            .collect::<Vec<Enemy>>();

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

        tavern_deck.shuffle(&mut rng);

        let tavern_deck = VecDeque::from(tavern_deck);

        // We can at most have all the cards in the deck in the discard pile
        let discard_pile = Vec::with_capacity(tavern_deck.len() + castle_deck.len());
        let attack_cards = vec![];
        Self {
            castle_deck,
            tavern_deck,
            discard_pile,
            attack_cards,
        }
    }

    pub fn draw_cards(&mut self, n_cards: u8) -> Vec<Card> {
        (0..n_cards)
            .filter_map(|_| self.tavern_deck.pop_front())
            .collect()
    }

    pub fn discard_card(&mut self, card: Card) {
        self.discard_pile.push(card);
    }

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

    pub fn get_current_enemy(&mut self) -> Option<&mut Enemy> {
        self.castle_deck.first_mut()
    }
}
