use crate::card::{Card, CardSuit, CardValue};
use crate::game::{Action, GameStatus};
use crate::player::{Player, PlayerId};
use crate::table::Table;
use itertools::Itertools;
use std::error::Error;
use std::fmt::Formatter;
use std::ops::Mul;

#[derive(Debug, Clone)]
pub struct State {
    table: Table,
    pub players: Vec<Player>,
    has_turn: PlayerId,
    times_yielded: usize,
    max_hand_size: u8,
}

#[derive(Debug)]
pub enum StateError {
    WrongNumberOfPlayers,
    NotAnEnemy(Card),
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?})", self))
    }
}

impl Error for StateError {}

impl State {
    pub fn new(n_players: i8) -> Result<Self, StateError> {
        let (n_jesters, max_hand_size) = match n_players {
            1 => (0, 8),
            2 => (0, 7),
            3 => (1, 6),
            4 => (2, 5),
            _ => return Err(StateError::WrongNumberOfPlayers),
        };
        let mut table = Table::new(n_jesters);
        let players = (0..n_players)
            .map(|id| Player::new(id, table.draw_cards(max_hand_size)))
            .collect();

        Ok(Self {
            table,
            players,
            has_turn: PlayerId(0),
            times_yielded: 0,
            max_hand_size,
        })
    }

    pub fn take_action(&self, action: &Action) -> GameStatus {
        let mut next_state = self.clone();
        Self::apply_action(&mut next_state, action);
        GameStatus::InProgress(next_state)
    }

    fn apply_action(&mut self, action: &Action) {
        // Step 1: Play a card from hand to attack the enemy
        let cards = match &action {
            Action::Play(c) => vec![c],
            Action::AnimalCombo(c1, c2) => vec![c1, c2],
            Action::Combo2(c1, c2) => vec![c1, c2],
            Action::Combo3(c1, c2, c3) => vec![c1, c2, c3],
            Action::Combo4(c1, c2, c3, c4) => vec![c1, c2, c3, c4],
            Action::Yield => vec![],
        };
        let mut attack_value: u16 = cards.iter().map(|c| c.attack_value()).sum();

        // TODO: Handle yielding

        // Step 2: Activate the played card’s suit power

        let suits: Vec<CardSuit> = cards.iter().map(|c| c.suit).collect();

        // Shield against enemy attack: During Step 4, reduce the attack
        // value of the current enemy by the attack value played. The shield effects
        // of spades are cumulative for all spades played against this enemy by any
        // player, and remain in effect until the enemy is defeated.
        if suits.contains(&CardSuit::Spades) {
            if let Some(enemy) = self.table.get_current_enemy() {
                enemy.decrease_attack(attack_value);
            }
        }

        // Heal from the discard: Shuffle the discard pile then count
        // out a number of cards facedown equal to the attack value played. Place
        // them under the Tavern deck (no peeking!) then, return the discard pile
        // to the table, faceup.
        if suits.contains(&CardSuit::Hearts) {
            self.table.heal_from_discard(attack_value as u8)
        }

        // The current player draws a card. The other players follow
        // in clockwise order drawing one card at a time until a number of cards
        // equal to the attack value played have been drawn. Players that have
        // reached their maximum hand size are skipped. Players may never draw cards
        // over their maximum hand size. There is no penalty for failing to draw
        // cards from an empty Tavern deck.
        if suits.contains(&CardSuit::Diamonds) {
            for i in 0..attack_value {
                for offset in 0..self.players.len() as u16 {
                    let index = ((i + offset) % (self.players.len() - 1) as u16) as usize;
                    let player = self.players.get_mut(index).unwrap();
                    if (player.hand.len()) < self.max_hand_size as usize {
                        let mut card = self.table.draw_cards(1);
                        player.hand.append(&mut card);
                        break;
                    }
                }
            }
        }

        // From rules: Double damage: During Step 3, damage dealt by clubs counts
        // for double. E.g., The 8 of Clubs deals 16 damage.
        if suits.contains(&CardSuit::Clubs) {
            attack_value *= 2;
        };

        // Step 3: Deal damage and check to see if the enemy is defeated
        if let Some(enemy) = self.table.get_current_enemy() {
            enemy.take_damage(attack_value);

            use std::cmp::Ordering::*;
            let enemy_card = *enemy.card();
            match enemy.health().cmp(&0) {
                Less => {
                    self.table.discard_card(enemy_card);
                    self.table.discard_attack_cards();
                }
                Equal => {
                    self.table.add_to_top_of_tavern_deck(enemy_card);
                    self.table.discard_attack_cards();
                }
                Greater => {}
            }
        }

        // Step 4: Suffer damage from the enemy by discarding cards
    }

    fn current_player(&self) -> &Player {
        self.players.get(self.has_turn.0 as usize).unwrap()
    }

    pub fn get_action_space(&self) -> Vec<Action> {
        use crate::card::CardValue::*;

        let player = self.current_player();

        // Single card actions
        let mut actions: Vec<Action> = player.hand.iter().map(|card| Action::Play(*card)).collect();

        let animal_combos = player
            .hand
            .iter()
            .filter(|card1| card1.value == Ace)
            .flat_map(|card1| {
                player
                    .hand
                    .iter()
                    // This avoids duplicate actions with symmetric cards
                    .filter(move |card2| match card2.value {
                        Ace => card1.suit > card2.suit,
                        _ => true,
                    })
                    .filter(|card2| card2.value != Jester)
                    .map(move |card2| Action::AnimalCombo(*card1, *card2))
            });
        actions.extend(animal_combos);

        // Combos of 2, 3 and 4 cards repspectively
        for card_value in [Two, Three, Four, Five].iter() {
            let same_value_cards = player
                .hand
                .iter()
                .filter(|c| c.value == *card_value)
                .collect::<Vec<&Card>>();
            let combos = [2, 3, 4]
                .iter()
                .flat_map(|len| same_value_cards.iter().combinations(*len))
                .filter(|combo| combo.iter().map(|c| c.attack_value()).sum::<u16>() < 10)
                .map(|combo| match combo[..] {
                    [c1, c2] => Action::Combo2(**c1, **c2),
                    [c1, c2, c3] => Action::Combo3(**c1, **c2, **c3),
                    [c1, c2, c3, c4] => Action::Combo4(**c1, **c2, **c3, **c4),
                    _ => panic!("Combo of wrong size."),
                });
            actions.extend(combos);
        }

        // All players cannot yield consequtively
        if self.times_yielded < self.players.len() - 1 {
            actions.push(Action::Yield);
        }
        actions
    }
}
