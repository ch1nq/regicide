
use super::State;
use crate::game::{
    card::{CardSuit::*, CardValue::*, FromCardIter},
    enemy::Enemy,
    Action, Card, GameStatus, Hand,
};

#[test]
fn jester_removes_immunity() {
    let mut state = State::<3>::new(Some(1337)).unwrap();
    *state.table.current_enemy_mut().unwrap() = Enemy::new(Card::new(Clubs, Queen));

    assert_eq!(state.current_enemy().unwrap().health(), 30);
    state = match state.take_action(&Action::Play(Card::new(None, Jester))) {
        GameStatus::InProgress(state) => state,
        _ => panic!("Game should not have ended"),
    };
    assert_eq!(state.current_enemy().unwrap().health(), 30);

    state = match state.take_action(&Action::Play(Card::new(Clubs, Two))) {
        GameStatus::InProgress(state) => state,
        _ => panic!("Game should not have ended"),
    };
    assert_eq!(state.current_enemy().unwrap().health(), 26);
}

#[test]
fn enemies_are_immune() {
    let mut state = State::<3>::new(Some(1337)).unwrap();
    *state.table.current_enemy_mut().unwrap() = Enemy::new(Card::new(Clubs, Queen));

    assert_eq!(state.current_enemy().unwrap().health(), 30);
    state = match state.take_action(&Action::Play(Card::new(Clubs, Two))) {
        GameStatus::InProgress(state) => state,
        _ => panic!("Game should not have ended"),
    };
    assert_eq!(state.current_enemy().unwrap().health(), 28);
}

const SEED: u64 = 1337;

macro_rules! hand {
        ($(($suit:expr, $value:expr)),+ $(,)?) => (
            Hand::from_card_iter(vec![
                $(Card::new($suit, $value)),+
            ].into_iter())
        );
    }

#[test]
fn no_duplicate_animal_combos() {
    let mut state = crate::game::state::State::<1>::new(Some(SEED)).unwrap();
    state.players[0].hand = hand!(
        (Diamonds, Ace),
        (Hearts, Ace),
        (Spades, Ace),
        (Spades, Six),
        (Spades, Seven),
    );
    let actions = state.get_action_space();
    dbg!(&state.players[0].hand);
    dbg!(&actions);
    assert_eq!(combo_count(&actions, "animal", 2), 9);
}

/// `n` is the size of the combos to be counted.
/// E.g. setting `n=2` will return the amount Combo2 actions.
fn combo_count(actions: &Vec<Action>, variant: &str, combo_len: usize) -> usize {
    actions
        .iter()
        .filter(|action| match (variant, action) {
            ("animal", Action::AnimalCombo(_, _)) => true,
            ("combo", Action::Combo(a)) if a.len() == combo_len => true,
            _ => false,
        })
        .count()
}

#[test]
fn two_card_combos() {
    let mut state = crate::game::state::State::<1>::new(Some(SEED)).unwrap();
    state.players[0].hand = hand!((Diamonds, Two), (Clubs, Two));
    let actions = state.get_action_space();
    dbg!(&state.players[0].hand);
    dbg!(&actions);
    assert_eq!(combo_count(&actions, "combo", 2), 1);
    assert_eq!(combo_count(&actions, "combo", 3), 0);
    assert_eq!(combo_count(&actions, "combo", 4), 0);
}

#[test]
fn three_card_combos() {
    let mut state = crate::game::state::State::<1>::new(Some(SEED)).unwrap();
    state.players[0].hand = hand!((Diamonds, Two), (Clubs, Two), (Hearts, Two),);
    let actions = state.get_action_space();
    dbg!(&state.players[0].hand);
    dbg!(&actions);
    assert_eq!(combo_count(&actions, "combo", 2), 3);
    assert_eq!(combo_count(&actions, "combo", 3), 1);
    assert_eq!(combo_count(&actions, "combo", 4), 0);
}

#[test]
fn four_card_combos() {
    let mut state = crate::game::state::State::<1>::new(Some(SEED)).unwrap();
    state.players[0].hand = hand!((Diamonds, Two), (Clubs, Two), (Hearts, Two), (Spades, Two),);
    let actions = state.get_action_space();
    dbg!(&state.players[0].hand);
    dbg!(&actions);
    assert_eq!(combo_count(&actions, "combo", 2), 6);
    assert_eq!(combo_count(&actions, "combo", 3), 4);
    assert_eq!(combo_count(&actions, "combo", 4), 1);
}
