use crate::game::policy::MyPolicy;
use crate::PyState;
use crate::{
    game::{
        state::{EmptyTable, MyEvaluator, MyMCTS, State},
        GameResult,
    },
    Action, PyAction,
};
use itertools::Itertools;
use mcts::MCTSManager;
use pyo3::prelude::*;

use super::Play;

#[derive(Clone)]
#[pyclass]
pub struct MCTSPlayer {
    playouts: u32,
    num_threads: usize,
    use_heuristics: bool,
    policy_variation: Option<u8>,
    ranked_actions: Option<Vec<(Action, u64, f64)>>,
}

#[pymethods]
impl MCTSPlayer {
    #[new]
    fn new(
        playouts: u32,
        num_threads: usize,
        use_heuristics: bool,
        policy_variation: Option<u8>,
    ) -> Self {
        Self {
            playouts,
            num_threads,
            use_heuristics,
            policy_variation,
            ranked_actions: None,
        }
    }

    /// Choose an action based on the given state
    fn play(&mut self, state: PyState) -> PyAction {
        self.play_py(state)
    }

    /// List actions with associated stats, sorted by most visits in the MCTS.
    /// The first Action in the list is considered the best move to play, and is
    /// also what `play()` will return.
    ///
    /// Updates on calls to `play()`.
    ///
    /// # Returns
    /// List of tuples in the form `(action, visits, avg_reward)`
    fn ranked_actions(&self) -> Vec<(PyAction, u64, f64)> {
        match &self.ranked_actions {
            Some(vec) => vec.iter().map(|&a| (a.0.into(), a.1, a.2)).collect(),
            None => vec![],
        }
    }
}

impl Play for MCTSPlayer {
    fn play<const N_PLAYERS: usize>(&mut self, state: State<N_PLAYERS>) -> Action {
        match self.use_heuristics {
            true => self.play_generic::<N_PLAYERS, true>(state),
            false => self.play_generic::<N_PLAYERS, false>(state),
        }
    }
}

impl MCTSPlayer {
    fn play_generic<const N_PLAYERS: usize, const USE_HEURISTICS: bool>(
        &mut self,
        state: State<N_PLAYERS>,
    ) -> Action {
        let policy = match self.policy_variation {
            None | Some(0) => MyPolicy::UCTBase {
                exploration_constant: GameResult::max_score() as f64,
            },
            Some(2) => MyPolicy::UCTVariation2 {
                max_score: GameResult::max_score() as f64,
                delta: 1e-3,
            },
            Some(3) => MyPolicy::UCTVariation3 {
                max_score: GameResult::max_score() as f64,
                delta: 1e-3,
            },
            Some(4) => MyPolicy::UCTVariation4 {
                max_score: GameResult::max_score() as f64,
            },
            Some(num) => panic!("Could not determine policy from number '{num}'"),
        };

        let mut mcts = MCTSManager::new(
            state,
            MyMCTS::<N_PLAYERS, USE_HEURISTICS>,
            MyEvaluator,
            policy,
            EmptyTable,
        );
        mcts.playout_n_parallel(self.playouts, self.num_threads);

        let root = mcts.tree().root_node();
        let mut actions = root.moves().into_iter().collect_vec();
        actions.sort_by_key(|action| -action.sum_rewards());
        actions.sort_by_key(|action| -(action.visits() as i64));

        // Store ranked moves
        self.ranked_actions = Some(
            actions
                .iter()
                .map(|a| {
                    (
                        *a.get_move(),
                        a.visits(),
                        a.sum_rewards() as f64 / a.visits() as f64,
                    )
                })
                .collect(),
        );

        *actions[0].get_move()
    }
}
