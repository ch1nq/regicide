use crate::card::{Card, CardValue};
use crate::game::{Action, GameStatus};
use crate::player::{Player, PlayerId};
use crate::table::Table;
use itertools::Itertools;
use std::error::Error;
use std::fmt::Formatter;

#[derive(Debug)]
pub struct State {
    table: Table,
    pub players: Vec<Player>,
    has_turn: PlayerId,
    shield: u16,
    times_yielded: usize,
}

#[derive(Debug)]
pub enum StateError {
    WrongNumberOfPlayers,
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
            shield: 0,
            times_yielded: 0,
        })
    }

    pub fn take_action(&self, _action: &Action) -> GameStatus {
        todo!()
    }

    fn current_player(&self) -> &Player {
        self.players.get(self.has_turn.0 as usize).unwrap()
    }

    pub fn get_action_space(&self) -> Vec<Action> {
        let player = self.current_player();

        // Single card actions
        let mut actions: Vec<Action> = player.hand.iter().map(|card| Action::Play(*card)).collect();

        // Animal combos
        actions.extend(
            player
                .hand
                .iter()
                .filter(|card1| card1.value == CardValue::Ace)
                .flat_map(|card1| {
                    player
                        .hand
                        .iter()
                        .filter(move |card2| card2 != &card1)
                        .filter(|card2| card2.value != CardValue::Jester)
                        .map(move |card2| Action::AnimalCombo(*card1, *card2))
                }),
            // BUG: If there are two Aces in the players hand, we
            // get two of the same Actions, only with swapped cards.
        );

        // Combos
        let valid_combo_cards = player
            .hand
            .iter()
            .filter(|card| match card.value {
                CardValue::Two | CardValue::Three | CardValue::Four | CardValue::Five => true,
                _ => false,
            })
            .unique_by(|card| card.value)
            .collect::<Vec<&Card>>();

        for (i, card) in valid_combo_cards.iter().enumerate() {
            let others = valid_combo_cards[i..]
                .iter()
                .filter(|c| c.value == card.value)
                .collect::<Vec<&&Card>>();
            let combos = [2, 3, 4]
                .iter()
                .flat_map(|len| others.iter().combinations(*len))
                .filter(|combo| combo.iter().map(|c| c.attack_value()).sum::<u16>() < 10)
                .map(|combo| match combo[..] {
                    [c1, c2] => Action::Combo2(***c1, ***c2),
                    [c1, c2, c3] => Action::Combo3(***c1, ***c2, ***c3),
                    [c1, c2, c3, c4] => Action::Combo4(***c1, ***c2, ***c3, ***c4),
                    _ => panic!("Combo of wrong size."),
                });
            // BUG: A player can get multiple of the same combinations. I am not exactly sure why.
            // A solution might be to simply filter out duplicates after all actions have been added,
            // but it would be better to fix the root of the problem.
            actions.extend(combos);
        }

        // All players cannot yield consequtively
        if self.times_yielded < self.players.len() - 1 {
            actions.push(Action::Yield);
        }
        actions
    }
}
