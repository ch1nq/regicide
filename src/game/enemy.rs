use super::card::{AttackValue, Card, CardValue};
use pyo3::prelude::*;
use std::convert::TryInto;

#[pyclass]
#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub struct Enemy {
    card: Card,
    health: i8,
    attack: u8,
    jester_applied: bool,
}

impl Enemy {
    pub fn new(card: Card) -> Enemy {
        use CardValue::*;
        let health = match card.value {
            Jack => 20,
            Queen => 30,
            King => 40,
            _ => panic!("{:?} is not an enemy", card),
        };
        Self {
            card,
            health,
            attack: card.attack_value() as u8,
            jester_applied: false,
        }
    }

    pub fn attack_value(&self) -> u8 {
        self.attack
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

    pub fn apply_jester(&mut self) {
        self.jester_applied = true;
    }

    pub fn jester_applied(&self) -> bool {
        self.jester_applied
    }
}

#[pymethods]
impl Enemy {
    fn __str__(&self) -> String {
        format!("{:?}", self)
    }
}
