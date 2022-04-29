use crate::game::{state::State, Action};

pub trait Play {
    fn play<const N: usize>(&mut self, state: State<N>) -> Action;
}

pub mod input_player;
pub mod mcts_player;
pub mod random_player;
