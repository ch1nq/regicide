use super::card::{AttackSum, Card, CardSuit, CardValue};
use super::enemy::Enemy;
use super::player::{Player, PlayerId};
use super::table::Table;
use crate::error::RegicideError;
use crate::game::{Action, GameResult, GameStatus};
use itertools::Itertools;
use rand::prelude::SliceRandom;
use rand::rngs::ThreadRng;
use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Hash)]
pub struct State {
    table: Table,
    pub players: Vec<Player>,
    has_turn: PlayerId,
    times_yielded: usize,
    max_hand_size: u8,
    action_type: ActionType,
    has_ended: Option<GameResult>,
    level: u8,
}

#[derive(Debug, Clone, PartialEq, Hash)]
enum ActionType {
    PlayCards,
    Discard(u8),
    Jester,
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
            action_type: ActionType::PlayCards,
            has_ended: None,
            level: 0,
        })
    }

    pub fn has_turn(&self) -> PlayerId {
        self.has_turn
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

                self.action_type = ActionType::PlayCards;
                self.next_player();
                GameStatus::InProgress(self)
            }
            Action::Yield => {
                if self.times_yielded < self.players.len() {
                    self.play_cards(vec![])
                } else {
                    // All players cannot yield consequtively
                    GameStatus::HasEnded(GameResult::Lost(self.reward()))
                }
            }
            Action::ChangePlayer(id) => {
                self.has_turn = *id;
                self.action_type = ActionType::PlayCards;
                GameStatus::InProgress(self)
            }
        }
    }

    fn play_cards(mut self, cards: Vec<Card>) -> GameStatus {
        use super::card::CardSuit::*;

        // Step 1: Play a card from hand to attack the enemy
        let mut attack_value: u16 = cards.attack_sum();

        // Apply jester if played
        if cards.contains(&Card::new(None, CardValue::Jester)) {
            let prior_spades_played = self
                .table
                .attack_cards()
                .iter()
                .filter(|card| card.suit == Spades)
                .collect_vec()
                .attack_sum();
            if let Some(enemy) = self.table.current_enemy_mut() {
                // If the Jester is played against a spades enemy,
                // spades played prior to the Jester will begin reducing
                // the attack value of the enemy
                if !enemy.jester_applied() && enemy.card().suit == Spades {
                    enemy.decrease_attack(prior_spades_played);
                }
                enemy.apply_jester();
            }
            self.action_type = ActionType::Jester;
        }

        self.current_player_mut().remove_from_hand(&cards);
        self.table.add_attack_cards(&cards);

        // Step 2: Activate the played card’s suit power

        let suits: Vec<CardSuit> = cards.iter().map(|c| c.suit).collect();
        let enemy_suit = match self.current_enemy() {
            Some(enemy) => enemy.card().suit,
            _ => CardSuit::None,
        };
        let jester_applied = match self.current_enemy() {
            Some(enemy) => enemy.jester_applied(),
            // TODO: Hmm, why would a jester be applied just becease there is no enemy?
            _ => true,
        };

        for suit in [Spades, Hearts, Diamonds, Clubs].iter() {
            if suits.contains(suit) && (&enemy_suit != suit || jester_applied) {
                match suit {
                    // Shield against enemy attack: During Step 4, reduce the attack
                    // value of the current enemy by the attack value played. The shield effects
                    // of spades are cumulative for all spades played against this enemy by any
                    // player, and remain in effect until the enemy is defeated.
                    Spades => {
                        if let Some(enemy) = self.table.current_enemy_mut() {
                            enemy.decrease_attack(attack_value);
                        }
                    }

                    // Heal from the discard: Shuffle the discard pile then count
                    // out a number of cards facedown equal to the attack value played. Place
                    // them under the Tavern deck (no peeking!) then, return the discard pile
                    // to the table, faceup.
                    Hearts => self.table.heal_from_discard(attack_value as u8),

                    // Draw cards: The current player draws a card. The other players follow
                    // in clockwise order drawing one card at a time until a number of cards
                    // equal to the attack value played have been drawn. Players that have
                    // reached their maximum hand size are skipped. Players may never draw cards
                    // over their maximum hand size. There is no penalty for failing to draw
                    // cards from an empty Tavern deck.
                    Diamonds => {
                        for i in 0..attack_value {
                            for offset in 0..self.players.len() as u16 {
                                let index = ((i + offset) % self.players.len() as u16) as usize;
                                let player = self.players.get_mut(index).unwrap();
                                if (player.hand.len()) < self.max_hand_size as usize {
                                    let mut card = self.table.draw_cards(1);
                                    // println!("Draw {:?} => {:?}", &card, &player);
                                    player.hand.append(&mut card);
                                    break;
                                }
                            }
                        }
                    }

                    // Double damage: During Step 3, damage dealt by clubs counts
                    // for double. E.g., The 8 of Clubs deals 16 damage.
                    Clubs => {
                        attack_value *= 2;
                    }
                    _ => {}
                }
            }
        }

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
                        self.level += 1;
                        GameStatus::InProgress(self)
                    }
                    Ordering::Equal => {
                        self.table.add_to_top_of_tavern_deck(enemy_card);
                        self.table.discard_attack_cards();
                        self.table.next_enemy();
                        self.level += 1;
                        GameStatus::InProgress(self)
                    }
                    Ordering::Greater => {
                        // Step 4: Suffer damage from the enemy by discarding cards
                        let enemy_attack = enemy.attack_value();
                        let player_health = self.current_player().total_hand_value();

                        if self.action_type == ActionType::Jester {
                            GameStatus::InProgress(self)
                        } else if enemy_attack as u16 > player_health {
                            GameStatus::HasEnded(GameResult::Lost(self.reward()))
                        } else {
                            // Player only needs to discard if they take damage
                            self.action_type = match enemy_attack {
                                0 => {
                                    self.next_player();
                                    ActionType::PlayCards
                                }
                                _ => ActionType::Discard(enemy_attack),
                            };
                            GameStatus::InProgress(self)
                        }
                    }
                }
            }
            Option::None => GameStatus::HasEnded(GameResult::Won),
        }
    }

    fn next_player(&mut self) {
        self.has_turn = self.has_turn.next_id(self.players.len());
    }

    pub fn reward(&self) -> u8 {
        match self.level {
            0..=4 => self.level * 2,
            5..=8 => 8 + (self.level - 4) * 2,
            _ => 20 + (self.level - 8) * 4,
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

        match self.action_type {
            ActionType::Discard(amount) => self.discard_actions(player, amount),
            ActionType::PlayCards => self.attack_actions(player),
            ActionType::Jester => self.jester_actions(),
        }
    }

    fn jester_actions(&self) -> Vec<Action> {
        self.players
            .iter()
            .map(|p| Action::ChangePlayer(p.id()))
            .collect()
    }

    fn discard_actions(&self, player: &Player, discard_amount: u8) -> Vec<Action> {
        let actions = (0..player.hand.len())
            .flat_map(|n_cards| {
                player
                    .hand
                    .iter()
                    .combinations(n_cards + 1)
                    .filter(|cards| {
                        let card_sum = cards.attack_sum();

                        card_sum >= discard_amount as u16
                    })
                    .map(|cards| cards.iter().map(|c| **c).collect())
                    .map(|cards| Action::Discard(cards))
            })
            .collect();
        actions
    }

    fn attack_actions(&self, player: &Player) -> Vec<Action> {
        use super::card::CardValue::*;

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
                .filter(|combo| combo.iter().map(|c| **c).collect_vec().attack_sum() <= 10)
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

    pub fn random_permutation(&self, rng: &mut ThreadRng) -> State {
        let mut new_state = self.clone();
        let player_hands = new_state
            .players
            .iter_mut()
            .filter(|player| player.id() != self.current_player().id())
            .map(|player| &mut player.hand)
            .collect_vec();

        new_state.table.permute(player_hands, rng);

        new_state
    }
}

use mcts::{CycleBehaviour, Evaluator, GameState, MoveEvaluation, SearchHandle, MCTS};

impl GameState for State {
    type Move = Action;
    type Player = Player;
    type MoveList = Vec<Action>;

    fn current_player(&self) -> Self::Player {
        self.current_player().clone()
    }
    fn available_moves(&self) -> Vec<Self::Move> {
        self.get_action_space()
    }
    fn make_move(&mut self, action: &Self::Move) {
        match self.clone().apply_action(action) {
            GameStatus::InProgress(state) => {
                *self = state;
            }
            GameStatus::HasEnded(result) => {
                self.has_ended = Some(result);
            }
        };
    }
}

impl TranspositionHash for State {
    fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        Hash::hash(&self, &mut hasher);
        hasher.finish()
    }
}

pub struct MyEvaluator;

impl Evaluator<MyMCTS> for MyEvaluator {
    type StateEvaluation = GameResult;

    // Random default policy
    fn evaluate_new_state(
        &self,
        state: &State,
        moves: &Vec<Action>,
        _: Option<SearchHandle<MyMCTS>>,
    ) -> (Vec<MoveEvaluation<MyMCTS>>, GameResult) {
        let mut node = state.clone();
        let mut rand = rand::thread_rng();
        let result;
        loop {
            let moves = node.available_moves();
            match moves.choose(&mut rand) {
                Some(random_action) => match node.apply_action(random_action) {
                    GameStatus::InProgress(state) => {
                        if let Some(res) = state.has_ended {
                            result = res;
                            break;
                        } else {
                            node = state.random_permutation(&mut rand);
                        }
                    }
                    GameStatus::HasEnded(res) => {
                        result = res;
                        break;
                    }
                },
                None => {
                    result = GameResult::Lost(node.reward());
                    break;
                }
            }
        }
        (vec![(); moves.len()], result)
    }
    fn evaluate_existing_state(
        &self,
        _: &State,
        evaln: &GameResult,
        _: SearchHandle<MyMCTS>,
    ) -> GameResult {
        *evaln
    }
    fn interpret_evaluation_for_player(&self, evaln: &GameResult, _player: &Player) -> i64 {
        match evaln {
            GameResult::Won => 36_i64,
            GameResult::Lost(reward) => *reward as i64,
        }
    }
}
use mcts::transposition_table::{ApproxTable, TranspositionHash};
use mcts::tree_policy::UCTPolicy;

#[derive(Default)]
pub struct MyMCTS;

impl MCTS for MyMCTS {
    type State = State;
    type Eval = MyEvaluator;
    type NodeData = ();
    type ExtraThreadData = ();
    type TreePolicy = UCTPolicy;
    type TranspositionTable = ApproxTable<Self>;

    fn cycle_behaviour(&self) -> CycleBehaviour<Self> {
        CycleBehaviour::UseCurrentEvalWhenCycleDetected
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // self.table: Table
        // self.pub players: Vec<Player>
        // self.has_turn: PlayerId
        // self.times_yielded: usize
        // self.max_hand_size: u8
        // self.action_type: ActionType
        write!(
            f,
            "{}{}{}",
            format!("Player: {:?}\n", self.has_turn()),
            format!("Hand:   {:?}\n", self.current_player().hand),
            format!("Enemy:  {:?}", self.current_enemy()),
            // format!("{}", self.table),
        )
    }
}
