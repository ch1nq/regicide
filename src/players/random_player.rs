use crate::{game::state::State, Action};
use pyo3::prelude::*;
use rand::{
    prelude::{SliceRandom, StdRng},
    SeedableRng,
};

use super::Play;

#[derive(Clone)]
#[pyclass]
pub struct RandomPlayer {
    rng: StdRng,
}

impl Play for RandomPlayer {
    fn play<const N: usize>(&mut self, state: State<N>) -> Action {
        *state.get_action_space().choose(&mut self.rng).unwrap()
    }
}

#[pymethods]
impl RandomPlayer {
    #[new]
    fn new(seed: Option<u64>) -> Self {
        let rng = match seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::from_rng(rand::thread_rng()).unwrap(),
        };
        Self { rng }
    }
}
