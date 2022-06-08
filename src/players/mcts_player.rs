use std::collections::HashMap;

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
use rand::SeedableRng;

use super::Play;

type Visits = u64;
type SumRewards = u64;
type AvgRewards = f64;

#[derive(Clone)]
#[pyclass]
pub struct MCTSPlayer {
    playouts: u32,
    deterministic_samples: u32,
    num_threads: usize,
    use_heuristics: bool,
    policy_variation: Option<u8>,
    ranked_actions: Option<Vec<(Action, Visits, AvgRewards)>>,
}

#[pymethods]
impl MCTSPlayer {
    #[new]
    fn new(
        playouts: u32,
        num_threads: usize,
        use_heuristics: bool,
        policy_variation: Option<u8>,
        deterministic_samples: Option<u32>,
    ) -> Self {
        Self {
            playouts,
            deterministic_samples: deterministic_samples.unwrap_or(1),
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
    fn ranked_actions(&self) -> Vec<(PyAction, Visits, AvgRewards)> {
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
                exploration_constant: 2_f64.sqrt() * GameResult::max_score() as f64,
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

        let mut meta_actions: HashMap<Action, (Visits, SumRewards)> = HashMap::new();

        let mut rng = rand::rngs::StdRng::from_rng(rand::thread_rng()).unwrap();
        for _ in 0..self.deterministic_samples {
            let permuted_state = state.random_permutation(&mut rng);
            let mut mcts = MCTSManager::new(
                permuted_state,
                // state,
                MyMCTS::<N_PLAYERS, USE_HEURISTICS>,
                MyEvaluator,
                policy.clone(),
                EmptyTable,
            );
            mcts.playout_n_parallel(self.playouts, self.num_threads);
            let root = mcts.tree().root_node();

            for move_info in root.moves().into_iter() {
                let action = move_info.get_move();
                let (visits, sum_rewards) = meta_actions.entry(*action).or_insert((0, 0));
                *visits += move_info.visits();
                *sum_rewards += move_info.sum_rewards() as SumRewards;
            }
        }

        let actions = meta_actions
            .iter()
            .sorted_by_key(|(_, (_, sum_rewards))| -(*sum_rewards as i64))
            .sorted_by_key(|(_, (visits, _))| -(*visits as i64))
            .collect_vec();

        let best_action = actions
            .get(0)
            .expect("No actions available to choose from")
            .0;

        // Store ranked moves
        self.ranked_actions = Some(
            actions
                .into_iter()
                .map(|(&action, &(visits, sum_rewards))| {
                    (action, visits, sum_rewards as f64 / visits as f64)
                })
                .collect(),
        );

        *best_action
    }
}
