use super::card::{Card, CardSuit, CardValue, CardVec, Hand};
use super::enemy::Enemy;
use crate::game::card::FromCardIter;
use arrayvec::ArrayVecCopy;
use itertools::Itertools;
use rand::prelude::SliceRandom;
use rand::rngs::StdRng;

#[derive(Debug, Clone, Copy, Hash)]
pub struct Table {
    castle_deck: ArrayVecCopy<Enemy, 12>,
    tavern_deck: CardVec,
    discard_pile: CardVec,
    attack_cards: CardVec,
}

impl Table {
    pub fn new(n_jesters: usize, rng: &mut StdRng) -> Self {
        let castle_deck = Self::new_castle_deck(rng);
        let tavern_deck = Self::new_tavern_deck(rng, n_jesters);
        let attack_cards = CardVec::new();
        let discard_pile = CardVec::new();

        Table {
            castle_deck,
            tavern_deck,
            discard_pile,
            attack_cards,
        }
    }

    fn new_castle_deck(rng: &mut StdRng) -> ArrayVecCopy<Enemy, 12> {
        IntoIterator::into_iter(CardValue::royals())
            .flat_map(|value| {
                let mut level = IntoIterator::into_iter(CardSuit::all())
                    .map(|suit| Enemy::new(Card::new(suit, value)))
                    .collect_vec();
                level.shuffle(rng);
                level
            })
            .collect()
    }

    fn new_tavern_deck(rng: &mut StdRng, n_jesters: usize) -> CardVec {
        let mut tavern_deck = CardValue::numbers()
            .iter()
            .flat_map(|value| {
                CardSuit::all()
                    .iter()
                    .map(|suit| Card::new(*suit, *value))
                    .collect::<Vec<_>>()
            })
            .chain(std::iter::repeat(Card::new(CardSuit::None, CardValue::Jester)).take(n_jesters))
            .collect::<CardVec>();
        tavern_deck.shuffle(rng);
        tavern_deck
    }

    pub fn draw_cards(&mut self, n_cards: usize) -> Hand {
        let mut cards = Hand::new();
        for card in (0..n_cards).filter_map(|_| self.tavern_deck.pop()) {
            cards.push(card);
        }
        cards
    }

    pub fn draw_card(&mut self) -> Option<Card> {
        self.tavern_deck.pop() // pop top of deck
    }

    pub fn discard_card(&mut self, card: Card) {
        self.discard_pile.push(card);
    }

    pub fn discard_cards(&mut self, cards: Hand) {
        self.discard_pile.extend(cards);
    }

    pub fn add_attack_cards<T>(&mut self, cards: T)
    where
        T: IntoIterator<Item = Card> + Sized,
    {
        self.attack_cards.extend(cards.into_iter())
    }

    pub fn attack_cards(&self) -> &CardVec {
        &self.attack_cards
    }

    /// Place all cards played by players against the enemy in the discard pile.
    pub fn discard_attack_cards(&mut self) {
        self.discard_pile.extend(self.attack_cards.drain(..));
    }

    pub fn add_to_top_of_tavern_deck(&mut self, card: Card) {
        self.tavern_deck.push(card); // push top
    }

    pub fn heal_from_discard(&mut self, n_cards: usize, rng: &mut StdRng) {
        self.discard_pile.shuffle(rng);
        let iter = std::iter::from_fn(|| self.discard_pile.pop())
            .take(n_cards)
            .collect_vec();
        for card in iter {
            self.tavern_deck.insert(0, card);
        }
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

    pub fn permute<'a>(&mut self, player_hands: &'a mut [&'a mut Hand], rng: &mut StdRng) {
        // Shuffle castle deck
        self.castle_deck = Self::new_castle_deck(rng)
            .iter()
            .filter(|enemy| self.castle_deck.contains(enemy))
            .copied()
            .collect();

        // Combine all potentially unknown cards into a single pile:
        // hands + discard pile + tavern deck
        let discard_pile = self.discard_pile.clone();
        let player_cards = player_hands
            .into_iter()
            .flat_map(|hand| hand.iter())
            .copied()
            .collect();
        let tavern_deck = self.tavern_deck.iter().map(|c| *c).collect();
        let mut combined_cards =
            CardVec::from_card_iter([discard_pile, player_cards, tavern_deck].concat());
        combined_cards.shuffle(rng);

        // Distribute cards back out
        for hand in player_hands {
            **hand = combined_cards.drain(..hand.len()).collect();
        }
        self.tavern_deck = combined_cards.drain(..self.tavern_deck.len()).collect();
        self.discard_pile = combined_cards;
    }
}
