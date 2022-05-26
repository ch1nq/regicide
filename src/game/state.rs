use super::card::{AttackSum, Card, CardSuit, CardValue, FromCardIter, Hand};
use super::enemy::Enemy;
use super::player::{Player, PlayerId};
use super::policy::MyPolicy;
use super::table::Table;
use crate::error::RegicideError;
use crate::game::{Action, GameResult, GameStatus};
use arrayvec::ArrayVecCopy;
use itertools::Itertools;
use rand::prelude::{SliceRandom, StdRng};
use rand::{RngCore, SeedableRng};
use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::convert::TryInto;
use std::hash::{Hash, Hasher};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, Hash)]
pub struct State<const N_PLAYERS: usize> {
    table: Table,
    players: [Player; N_PLAYERS],
    has_turn: PlayerId,
    times_yielded: usize,
    max_hand_size: u8,
    action_type: ActionType,
    has_ended: Option<GameResult>,
    level: u8,
    rng_seed: u64,
    hand_refills_left: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
enum ActionType {
    PlayCards,
    Discard(u8),
    Jester,
}

impl<const N_PLAYERS: usize> State<N_PLAYERS> {
    fn new_rng(seed: Option<u64>) -> StdRng {
        match seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::from_rng(rand::thread_rng()).unwrap(),
        }
    }

    fn get_rng(&mut self) -> StdRng {
        let mut rng = Self::new_rng(Some(self.rng_seed));
        self.rng_seed = rng.next_u64();
        rng
    }

    pub fn new(seed: Option<u64>) -> Result<Self, RegicideError> {
        let mut rng = Self::new_rng(seed);
        let (n_jesters, max_hand_size, hand_refills_left) = match N_PLAYERS {
            1 => (0, 8, 2),
            2 => (0, 7, 0),
            3 => (1, 6, 0),
            4 => (2, 5, 0),
            _ => return Err(RegicideError::WrongNumberOfPlayers),
        };
        let mut table = Table::new(n_jesters, &mut rng);
        let players = (0..N_PLAYERS)
            .map(|id| Player::new(id, table.draw_cards(max_hand_size)))
            .collect_vec()
            .try_into()
            .unwrap();

        Ok(Self {
            table,
            players,
            has_turn: PlayerId(0),
            times_yielded: 0,
            max_hand_size: max_hand_size as u8,
            action_type: ActionType::PlayCards,
            has_ended: None,
            level: 0,
            rng_seed: rng.next_u64(),
            hand_refills_left,
        })
    }

    pub fn has_turn(&self) -> PlayerId {
        self.has_turn
    }

    pub fn current_hand(&self) -> Hand {
        self.current_player().hand
    }

    pub fn take_action(&self, action: &Action) -> GameStatus<N_PLAYERS> {
        let next_state = *self;
        Self::apply_action(next_state, action)
    }

    fn apply_action(mut self, action: &Action) -> GameStatus<N_PLAYERS> {
        self.times_yielded = match action {
            Action::Discard(_) | Action::RefillHand => self.times_yielded,
            Action::Yield => self.times_yielded + 1,
            _ => 0,
        };

        match action {
            Action::Play(c) => self.play_cards(Hand::from_card_iter([*c])),
            Action::AnimalCombo(c1, c2) => self.play_cards(Hand::from_card_iter([*c1, *c2])),
            Action::Combo(cards) => {
                self.play_cards(Hand::from_card_iter(cards.into_iter().copied()))
            }
            Action::Discard(discard_cards) => {
                self.current_player_mut().hand = self
                    .current_player_mut()
                    .hand
                    .iter()
                    .filter(|&card| !discard_cards.iter().contains(&card))
                    .copied()
                    .collect();
                self.table.discard_cards(*discard_cards);

                self.action_type = ActionType::PlayCards;
                self.next_player();
                GameStatus::InProgress(self)
            }
            Action::Yield => {
                if self.times_yielded < self.players.len() {
                    self.play_cards(Hand::new())
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
            Action::RefillHand => {
                self.hand_refills_left -= 1;
                let hand = self.current_player_mut().hand;
                self.table.discard_cards(hand);
                self.current_player_mut().hand = self.table.draw_cards(self.max_hand_size.into());
                GameStatus::InProgress(self)
            }
        }
    }

    fn play_cards<'a>(mut self, cards: Hand) -> GameStatus<N_PLAYERS> {
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
        self.table.add_attack_cards(cards.iter().copied());

        // Step 2: Activate the played card’s suit power

        let suits: Vec<_> = cards.iter().map(|c| c.suit).collect();
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
                    Hearts => {
                        let mut rng = self.get_rng();
                        self.table
                            .heal_from_discard(attack_value as usize, &mut rng)
                    }
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
                                    if let Some(card) = self.table.draw_card() {
                                        player.hand.push(card);
                                    }
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
        self.level
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

        let mut actions = match self.action_type {
            ActionType::Discard(amount) => self.discard_actions(player, amount),
            ActionType::PlayCards => self.attack_actions(player),
            ActionType::Jester => self.jester_actions(),
        };

        // RefillHand is only appriate to play in single player games, when
        // either the player is discarding or about to play card. Since there
        // are no jesters in the game when there is only one player, ActionType
        // can never be Jester and it is safe to assume that we either at the
        // Discard or PlayCards stage.
        if N_PLAYERS == 1 && self.hand_refills_left > 0 {
            actions.push(Action::RefillHand)
        }

        actions
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
                .copied()
                .collect::<Hand>();
            let combos = [2, 3, 4]
                .iter()
                .flat_map(|len| same_value_cards.iter().combinations(*len))
                .filter(|combo| {
                    combo
                        .into_iter()
                        .map(|c| **c)
                        .collect::<Hand>()
                        .attack_sum()
                        <= 10
                })
                .map(|combo| {
                    Action::Combo(ArrayVecCopy::<Card, 4>::from_card_iter(
                        combo.into_iter().copied(),
                    ))
                });
            actions.extend(combos);
        }

        actions.push(Action::Yield);
        actions
    }

    pub fn random_permutation(&self, rng: &mut StdRng) -> State<N_PLAYERS> {
        let mut new_state = *self;
        let mut player_hands = new_state
            .players
            .iter_mut()
            .filter(|player| player.id() != self.current_player().id())
            .map(|player| &mut player.hand)
            .collect_vec();
        new_state.table.permute(&mut player_hands[..], rng);

        new_state
    }
}

use mcts::{Evaluator, GameState, MoveEvaluation, SearchHandle, MCTS};

impl<const N_PLAYERS: usize> GameState for State<N_PLAYERS> {
    type Move = Action;
    type Player = Player;
    type MoveList = Vec<Action>;

    fn current_player(&self) -> Self::Player {
        *self.current_player()
    }
    fn available_moves(&self) -> Vec<Self::Move> {
        self.get_action_space()
    }
    fn make_move(&mut self, action: &Self::Move) {
        match (*self).apply_action(action) {
            GameStatus::InProgress(state) => {
                *self = state;
            }
            GameStatus::HasEnded(result) => {
                self.has_ended = Some(result);
            }
        };
    }
}

impl<const N_PLAYERS: usize> TranspositionHash for State<N_PLAYERS> {
    fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        Hash::hash(&self, &mut hasher);
        hasher.finish()
    }
}

pub struct MyEvaluator<const N_PLAYERS: usize>;

fn prune_bad_moves(all_moves: Vec<Action>) -> Vec<Action> {
    let filters = vec![
        // Never discard any Diamonds
        |m: &&Action| match m {
            Action::Discard(cards) => cards.iter().all(|card| match card {
                Card {
                    suit: CardSuit::Diamonds,
                    ..
                } => false,
                _ => true,
            }),
            _ => true,
        },
    ];
    let mut moves_iter = all_moves;
    for f in filters {
        let foo = moves_iter.iter().filter(f).copied().collect_vec();
        if foo.len() > 0 {
            moves_iter = foo;
        }
    }
    moves_iter
}

impl<const N_PLAYERS: usize, const HEURISTICS: bool> Evaluator<MyMCTS<N_PLAYERS, HEURISTICS>>
    for MyEvaluator<N_PLAYERS>
{
    type StateEvaluation = GameResult;

    // Random default policy
    fn evaluate_new_state(
        &self,
        state: &State<N_PLAYERS>,
        moves: &Vec<Action>,
        _: Option<SearchHandle<MyMCTS<N_PLAYERS, HEURISTICS>>>,
    ) -> (
        Vec<MoveEvaluation<MyMCTS<N_PLAYERS, HEURISTICS>>>,
        GameResult,
    ) {
        let mut node = *state;
        let mut rng = rand::rngs::StdRng::from_rng(rand::thread_rng()).unwrap();
        node.random_permutation(&mut rng);
        let result;
        loop {
            let mut moves = node.available_moves();
            if HEURISTICS {
                moves = prune_bad_moves(moves);
            }
            match moves.choose(&mut rng) {
                Some(random_action) => match node.apply_action(random_action) {
                    GameStatus::InProgress(new_state) => {
                        if let Some(res) = new_state.has_ended {
                            result = res;
                            break;
                        } else {
                            node = new_state;
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
        state: &State<N_PLAYERS>,
        _evaln: &GameResult,
        handle: SearchHandle<MyMCTS<N_PLAYERS, HEURISTICS>>,
    ) -> GameResult {
        self.evaluate_new_state(state, &state.available_moves(), Some(handle))
            .1
    }

    fn interpret_evaluation_for_player(&self, evaln: &GameResult, _player: &Player) -> i64 {
        match evaln {
            GameResult::Won => GameResult::max_score().into(),
            GameResult::Lost(reward) => (*reward).into(),
        }
    }
}

use mcts::transposition_table::{TranspositionHash, TranspositionTable};
use mcts::CycleBehaviour;

#[derive(Default)]
pub struct MyMCTS<const N_PLAYERS: usize, const HEURISTICS: bool>;

pub struct EmptyTable;

unsafe impl<Spec: MCTS> TranspositionTable<Spec> for EmptyTable {
    fn insert<'a>(
        &'a self,
        _key: &<Spec as MCTS>::State,
        _value: &'a mcts::SearchNode<Spec>,
        _handle: SearchHandle<Spec>,
    ) -> Option<&'a mcts::SearchNode<Spec>> {
        None
    }

    fn lookup<'a>(
        &'a self,
        _key: &<Spec as MCTS>::State,
        _handle: SearchHandle<Spec>,
    ) -> Option<&'a mcts::SearchNode<Spec>> {
        None
    }
}

impl<const N_PLAYERS: usize, const HEURISTICS: bool> MCTS for MyMCTS<N_PLAYERS, HEURISTICS> {
    type State = State<N_PLAYERS>;
    type Eval = MyEvaluator<N_PLAYERS>;
    type NodeData = ();
    type ExtraThreadData = ();
    type TreePolicy = MyPolicy;
    type TranspositionTable = EmptyTable;

    fn max_playout_length(&self) -> usize {
        1_000
    }

    fn cycle_behaviour(&self) -> CycleBehaviour<Self> {
        CycleBehaviour::PanicWhenCycleDetected
    }

    fn visits_before_expansion(&self) -> u64 {
        1000
    }

    fn node_limit(&self) -> usize {
        std::usize::MAX
    }

    fn select_child_after_search<'a>(
        &self,
        children: &'a [mcts::MoveInfo<Self>],
    ) -> &'a mcts::MoveInfo<Self> {
        children
            .into_iter()
            .max_by_key(|child| child.visits())
            .unwrap()
    }

    fn on_backpropagation(
        &self,
        _evaln: &mcts::StateEvaluation<Self>,
        _handle: SearchHandle<Self>,
    ) {
    }
}

impl<const N_PLAYERS: usize> std::fmt::Display for State<N_PLAYERS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            format!("Player: {:?}\n", self.has_turn()),
            format!("Hand:   {:?}\n", self.current_player().hand),
            format!("Enemy:  {:?}", self.current_enemy()),
        )
    }
}
