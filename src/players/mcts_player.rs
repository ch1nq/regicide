use crate::{
    game::{
        state::{EmptyTable, MyEvaluator, MyMCTS, State},
        GameResult,
    },
    Action, PyAction,
};
use crate::{PyState, StateEnum};
use mcts::tree_policy::UCTPolicy;
use mcts::MCTSManager;
use pyo3::prelude::*;

use super::Play;

#[derive(Clone)]
#[pyclass]
pub struct MCTSPlayer {
    n_playouts: u32,
    use_heuristics: bool,
}

#[pymethods]
impl MCTSPlayer {
    #[new]
    fn new(n_playouts: u32, use_heuristics: bool) -> Self {
        Self {
            n_playouts,
            use_heuristics,
        }
    }

    #[pyo3(name = "play")]
    fn py_play(&mut self, state: PyState) -> PyAction {
        self.play_py(state)
    }
}

impl Play for MCTSPlayer {
    fn play<const N: usize>(&mut self, state: State<N>) -> Action {
        match self.use_heuristics {
            true => self.play_generic::<N, true>(state),
            false => self.play_generic::<N, false>(state),
        }
    }
}

impl MCTSPlayer {
    fn play_generic<const N: usize, const USE_HEURISTICS: bool>(
        &mut self,
        state: State<N>,
    ) -> Action {
        let mut mcts = MCTSManager::new(
            state,
            MyMCTS::<N, USE_HEURISTICS>,
            MyEvaluator,
            UCTPolicy::new(GameResult::max_score() as f64 / f64::sqrt(2_f64)),
            EmptyTable,
        );

        // println!("{}", state);

        mcts.playout_n_parallel(self.n_playouts, 8);

        // // Print top 10 moves
        // let root = mcts.tree().root_node();
        // let mut moves = root.moves().into_iter().collect_vec();
        // moves.sort_by_key(|x| -(x.visits() as i64));

        // println!();
        // print!("  ---->");
        // for mov in moves.iter().take(3) {
        //     println!("\t{:?}", mov);
        // }
        // println!();

        mcts.best_move().expect("There should be a best move")
    }
}
