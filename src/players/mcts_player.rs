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
}

#[pymethods]
impl MCTSPlayer {
    #[new]
    fn new(playouts: u32, num_threads: usize, use_heuristics: bool) -> Self {
        Self {
            playouts,
            num_threads,
            use_heuristics,
        }
    }

    #[pyo3(name = "play")]
    fn py_play(&mut self, state: PyState) -> PyAction {
        self.play_py(state)
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
        // let policy = crate::game::policy::MyPolicy::UCTBase {
        //     exploration_constant: GameResult::max_score() as f64 / f64::sqrt(2_f64),
        // };
        let policy = crate::game::policy::MyPolicy::UCTVariation4 {
            max_score: GameResult::max_score() as u64,
        };

        let mut mcts = MCTSManager::new(
            state,
            MyMCTS::<N_PLAYERS, USE_HEURISTICS>,
            MyEvaluator,
            policy,
            EmptyTable,
        );
        mcts.playout_n_parallel(self.playouts, self.num_threads);

        // Print top 10 moves
        let root = mcts.tree().root_node();
        let mut moves = root.moves().into_iter().collect_vec();
        moves.sort_by_key(|x| -(x.visits() as i64));

        println!();
        print!("  ---->");
        for mov in moves.iter().take(3) {
            println!("\t{:?}", mov);
        }
        println!();

        mcts.best_move().expect("There should be a best move")
    }
}
