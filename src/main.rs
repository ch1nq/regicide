use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::{RngCore, SeedableRng};
use regicide_rl::game::state::{self, EmptyTable};
use regicide_rl::game::{GameResult, GameStatus};
use std::io::stdin;

fn main() {
    let mut meta_rng = rand::rngs::StdRng::seed_from_u64(1337);
    let mut result = GameResult::Won;
    for _ in 0..1 {
        // result = _random_playout(Some(meta_rng.next_u64()));
        result = mcts_playout(Some(meta_rng.next_u64()));
    }
    dbg!(result);
}

fn _random_playout(seed: Option<u64>) -> GameResult {
    let mut rng = rand::rngs::StdRng::seed_from_u64(1337);
    let mut state = state::State::<3>::new(seed).unwrap();

    loop {
        let actions = state.get_action_space();
        let action = actions.choose(&mut rng).unwrap();
        match state.take_action(action) {
            GameStatus::InProgress(new_state) => {
                state = new_state;
            }
            GameStatus::HasEnded(result) => {
                return result;
            }
        };
    }
}

use mcts::tree_policy::UCTPolicy;
use mcts::MCTSManager;
use state::{MyEvaluator, MyMCTS};

fn mcts_playout(seed: Option<u64>) -> GameResult {
    let mut state = state::State::<1>::new(seed).unwrap();

    loop {
        let mut mcts = MCTSManager::new(
            state,
            MyMCTS,
            MyEvaluator,
            // UCTPolicy::new(4.4),
            UCTPolicy::new(36_f64 / f64::sqrt(2_f64)),
            // UCTPolicy::new(1_f64 / f64::sqrt(2_f64)),
            EmptyTable,
        );

        println!("{}", state);

        mcts.playout_n(1_000_000);
        // mcts.playout_n_parallel(10_000_000, 6);

        // Print top 10 moves
        let root = mcts.tree().root_node();
        let mut moves = root.moves().into_iter().take(10).collect_vec();
        moves.sort_by_key(|x| -(x.visits() as i64));
        println!();
        print!("--> ");
        for mov in moves {
            println!("{:?}", mov);
        }
        println!();

        match state.take_action(&mcts.best_move().expect("There should be a best move")) {
            GameStatus::InProgress(new_state) => {
                state = new_state;
            }
            GameStatus::HasEnded(result) => {
                return result;
            }
        };
    }
}

fn _input_playout() -> GameResult {
    let mut state = state::State::<3>::new(Some(1337)).unwrap();

    loop {
        let mut input = String::new();
        let actions = state.get_action_space();

        println!("{}", ["="; 60].concat());
        println!("Current player: {:?}", state.has_turn());
        println!("Current enemy: \n\n{:#?}\n", state.current_enemy());
        println!(
            "Available actions are: \n {} \n",
            &actions
                .iter()
                .enumerate()
                .map(|(i, action)| format!("{}: {:?}", i, action))
                .fold("".to_string(), |acc, s| acc + "\n" + &s)
        );
        println!("Please select an action:");

        stdin().read_line(&mut input).expect("Error reading input");
        if let Ok(idx) = input.trim().parse::<usize>() {
            if let Some(action) = actions.get(idx) {
                match state.take_action(action) {
                    GameStatus::InProgress(new_state) => {
                        state = new_state;
                    }
                    GameStatus::HasEnded(result) => {
                        return result;
                    }
                };
            } else {
                println!("Could not match input with any action. Please try again!");
            }
        } else {
            println!("failed to parse input {:?}", input);
        }
    }
}
