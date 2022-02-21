mod card;
mod game;
mod player;
mod state;
mod table;

use rand::seq::SliceRandom;
use rand::thread_rng;

fn main() {
    // test_multicombo_bug();
}

fn simulate_10_moves() {
    let state = state::State::new(2).unwrap();

    for _ in 0..10 {
        let mut rng = thread_rng();
        let actions = state.get_action_space();
        let action = actions.choose(&mut rng).unwrap();
        dbg!(action);
        let state = state.take_action(action);
    }
}

#[cfg(test)]
mod tests {
    use crate::card::Card;
    use crate::card::CardSuit::*;
    use crate::card::CardValue::*;
    use crate::game::Action;

    #[test]
    fn no_duplicate_animal_combos() {
        let mut state = crate::state::State::new(1).unwrap();
        state.players[0].hand = vec![
            Card::new(Diamonds, Ace),
            Card::new(Hearts, Ace),
            Card::new(Spades, Ace),
            Card::new(Spades, Six),
            Card::new(Spades, Seven),
        ];
        let actions = state.get_action_space();
        dbg!(&state.players[0].hand);
        dbg!(&actions);
        assert_eq!(combo_count(&actions, "a"), 9);
    }

    /// `n` is the size of the combos to be counted.
    /// E.g. setting `n=2` will return the amount Combo2 actions.
    fn combo_count(actions: &Vec<Action>, variant: &str) -> usize {
        actions
            .iter()
            .filter(|action| match (variant, action) {
                ("a", Action::AnimalCombo(_, _)) => true,
                ("c2", Action::Combo2(_, _)) => true,
                ("c3", Action::Combo3(_, _, _)) => true,
                ("c4", Action::Combo4(_, _, _, _)) => true,
                _ => false,
            })
            .count()
    }

    #[test]
    fn two_card_combos() {
        let mut state = crate::state::State::new(1).unwrap();
        state.players[0].hand = vec![Card::new(Diamonds, Two), Card::new(Clubs, Two)];
        let actions = state.get_action_space();
        dbg!(&state.players[0].hand);
        dbg!(&actions);
        assert_eq!(combo_count(&actions, "c2"), 1);
        assert_eq!(combo_count(&actions, "c3"), 0);
        assert_eq!(combo_count(&actions, "c4"), 0);
    }

    #[test]
    fn three_card_combos() {
        let mut state = crate::state::State::new(1).unwrap();
        state.players[0].hand = vec![
            Card::new(Diamonds, Two),
            Card::new(Clubs, Two),
            Card::new(Hearts, Two),
        ];
        let actions = state.get_action_space();
        dbg!(&state.players[0].hand);
        dbg!(&actions);
        assert_eq!(combo_count(&actions, "c2"), 3);
        assert_eq!(combo_count(&actions, "c3"), 1);
        assert_eq!(combo_count(&actions, "c4"), 0);
    }

    #[test]
    fn four_card_combos() {
        let mut state = crate::state::State::new(1).unwrap();
        state.players[0].hand = vec![
            Card::new(Diamonds, Two),
            Card::new(Clubs, Two),
            Card::new(Hearts, Two),
            Card::new(Spades, Two),
        ];
        let actions = state.get_action_space();
        dbg!(&state.players[0].hand);
        dbg!(&actions);
        assert_eq!(combo_count(&actions, "c2"), 6);
        assert_eq!(combo_count(&actions, "c3"), 4);
        assert_eq!(combo_count(&actions, "c4"), 1);
    }
}
