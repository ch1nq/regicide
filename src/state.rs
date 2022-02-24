use crate::card::{Card, CardSuit};
use crate::error::RegicideError;
use crate::game::{Action, GameResult, GameStatus};
use crate::player::{Player, PlayerId};
use crate::table::{Enemy, Table};
use itertools::Itertools;
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct State {
    table: Table,
    pub players: Vec<Player>,
    has_turn: PlayerId,
    times_yielded: usize,
    max_hand_size: u8,
    must_discard: Option<u8>,
}

impl State {
    pub fn new(n_players: usize) -> Result<Self, RegicideError> {
        let (n_jesters, max_hand_size) = match n_players {
            1 => (0, 8),
            2 => (0, 7),
            3 => (1, 6),
            4 => (2, 5),
            _ => return Err(RegicideError::WrongNumberOfPlayers),
        };
        let mut table = Table::new(n_jesters)?;
        let players = (0..n_players)
            .map(|id| Player::new(id, table.draw_cards(max_hand_size)))
            .collect();

        Ok(Self {
            table,
            players,
            has_turn: PlayerId(0),
            times_yielded: 0,
            max_hand_size,
            must_discard: None,
        })
    }

    pub fn take_action(&self, action: &Action) -> GameStatus {
        let next_state = self.clone();
        Self::apply_action(next_state, action)
    }

    fn apply_action(mut self, action: &Action) -> GameStatus {
        self.times_yielded = match action {
            Action::Discard(_) => self.times_yielded,
            Action::Yield => self.times_yielded + 1,
            _ => 0,
        };

        match action {
            Action::Play(c) => self.play_cards(vec![*c]),
            Action::AnimalCombo(c1, c2) => self.play_cards(vec![*c1, *c2]),
            Action::Combo2(c1, c2) => self.play_cards(vec![*c1, *c2]),
            Action::Combo3(c1, c2, c3) => self.play_cards(vec![*c1, *c2, *c3]),
            Action::Combo4(c1, c2, c3, c4) => self.play_cards(vec![*c1, *c2, *c3, *c4]),
            Action::Discard(discard_cards) => {
                self.current_player_mut().hand = self
                    .current_player_mut()
                    .hand
                    .iter()
                    .filter(|&card| !discard_cards.contains(card))
                    .map(|c| *c)
                    .collect();

                self.must_discard = None;
                self.has_turn = self.has_turn.next_id(self.players.len());
                GameStatus::InProgress(self)
            }
            Action::Yield => {
                if self.times_yielded < self.players.len() {
                    self.play_cards(vec![])
                } else {
                    // All players cannot yield consequtively
                    GameStatus::HasEnded(GameResult::Lost)
                }
            }
        }
    }

    fn play_cards(mut self, cards: Vec<Card>) -> GameStatus {
        use crate::card::CardSuit::*;

        // Step 1: Play a card from hand to attack the enemy
        let mut attack_value: u16 = cards.iter().map(|c| c.attack_value()).sum();

        self.table.add_attack_cards(&cards);

        // Step 2: Activate the played card’s suit power
        let suits: Vec<CardSuit> = cards.iter().map(|c| c.suit).collect();

        let enemy_suit = match self.current_enemy() {
            Some(enemy) => enemy.card().suit,
            _ => CardSuit::None,
        };

        // Shield against enemy attack: During Step 4, reduce the attack
        // value of the current enemy by the attack value played. The shield effects
        // of spades are cumulative for all spades played against this enemy by any
        // player, and remain in effect until the enemy is defeated.
        if suits.contains(&Spades) && enemy_suit != Spades {
            if let Some(enemy) = self.table.current_enemy_mut() {
                enemy.decrease_attack(attack_value);
            }
        }

        // Heal from the discard: Shuffle the discard pile then count
        // out a number of cards facedown equal to the attack value played. Place
        // them under the Tavern deck (no peeking!) then, return the discard pile
        // to the table, faceup.
        if suits.contains(&Hearts) && enemy_suit != Hearts {
            self.table.heal_from_discard(attack_value as u8)
        }

        // Draw cards: The current player draws a card. The other players follow
        // in clockwise order drawing one card at a time until a number of cards
        // equal to the attack value played have been drawn. Players that have
        // reached their maximum hand size are skipped. Players may never draw cards
        // over their maximum hand size. There is no penalty for failing to draw
        // cards from an empty Tavern deck.
        if suits.contains(&Diamonds) && enemy_suit != Diamonds {
            for i in 0..attack_value {
                for offset in 0..self.players.len() as u16 {
                    let index = ((i + offset) % self.players.len() as u16) as usize;
                    let player = self.players.get_mut(index).unwrap();
                    if (player.hand.len()) < self.max_hand_size as usize {
                        let mut card = self.table.draw_cards(1);
                        player.hand.append(&mut card);
                        break;
                    }
                }
            }
        }

        // Double damage: During Step 3, damage dealt by clubs counts
        // for double. E.g., The 8 of Clubs deals 16 damage.
        if suits.contains(&Clubs) && enemy_suit != Clubs {
            attack_value *= 2;
        };

        // Step 3: Deal damage and check to see if the enemy is defeated
        match self.table.current_enemy_mut() {
            Some(enemy) => {
                enemy.take_damage(attack_value);

                let enemy_card = *enemy.card();

                match enemy.health().cmp(&0) {
                    Ordering::Less => {
                        self.table.discard_card(enemy_card);
                        self.table.discard_attack_cards();
                        self.table.next_enemy();
                        GameStatus::InProgress(self)
                    }
                    Ordering::Equal => {
                        self.table.add_to_top_of_tavern_deck(enemy_card);
                        self.table.discard_attack_cards();
                        self.table.next_enemy();
                        GameStatus::InProgress(self)
                    }
                    Ordering::Greater => {
                        // Step 4: Suffer damage from the enemy by discarding cards
                        let enemy_attack = enemy.attack_value();
                        let player_health = self.current_player().total_hand_value();

                        if enemy_attack as u16 > player_health {
                            GameStatus::HasEnded(GameResult::Lost)
                        } else {
                            // Player only needs to discard if they take damage
                            self.must_discard = match enemy_attack {
                                0 => Option::None,
                                _ => Some(enemy_attack),
                            };
                            GameStatus::InProgress(self)
                        }
                    }
                }
            }
            Option::None => GameStatus::HasEnded(GameResult::Won),
        }
    }

    fn current_player_mut(&mut self) -> &mut Player {
        self.players.get_mut(self.has_turn.0).unwrap()
    }

    fn current_player(&self) -> &Player {
        self.players.get(self.has_turn.0).unwrap()
    }

    pub fn current_enemy(&self) -> Option<&Enemy> {
        self.table.current_enemy()
    }

    pub fn get_action_space(&self) -> Vec<Action> {
        let player = self.current_player();

        match self.must_discard {
            Some(discard_amount) => self.discard_actions(player, discard_amount),
            None => self.attack_actions(player),
        }
    }

    fn discard_actions(&self, player: &Player, discard_amount: u8) -> Vec<Action> {
        let actions = (0..player.hand.len())
            .flat_map(|n_cards| {
                player
                    .hand
                    .iter()
                    .combinations(n_cards + 1)
                    .filter(|cards| {
                        let card_sum = cards.iter().map(|c| c.attack_value()).sum::<u16>();

                        card_sum >= discard_amount as u16
                    })
                    .map(|cards| cards.iter().map(|c| **c).collect())
                    .map(|cards| Action::Discard(cards))
            })
            .collect();
        actions
    }

    fn attack_actions(&self, player: &Player) -> Vec<Action> {
        use crate::card::CardValue::*;

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

        actions.push(Action::Yield);
        actions
    }
}
