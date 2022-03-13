use rand::seq::SliceRandom;
use rand::thread_rng;
use regicide_rl::game::state;
use regicide_rl::game::{GameResult, GameStatus};
use std::io::stdin;

fn main() {
    let result = mcts_playout();
    dbg!(result);
}

fn _random_playout() -> GameResult {
    let mut rng = thread_rng();
    let mut state = state::State::new(3).unwrap();

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

use mcts::transposition_table::ApproxTable;
use mcts::tree_policy::UCTPolicy;
use mcts::MCTSManager;
use state::{MyEvaluator, MyMCTS};

fn mcts_playout() -> GameResult {
    let mut state = state::State::new(3).unwrap();

    loop {
        let mut mcts = MCTSManager::new(
            state.clone(),
            MyMCTS,
            MyEvaluator,
            // MyEvaluator(state.current_player()),
            // UCTPolicy::new(4.4),
            UCTPolicy::new(36_f64 / f64::sqrt(2_f64)),
            // ApproxTable::new(2.pow(11)),
            ApproxTable::new(2 << 24),
        );

        println!("{}", state);

        mcts.playout_n_parallel(10_000_000, 6);

        let resulting_action_info = mcts.principal_variation_info(1);
        let resulting_action = mcts.principal_variation(1);
        let action = resulting_action
            .first()
            .expect("Could not find action")
            .clone();

        if let Some(info) = resulting_action_info.first() {
            println!("{:?}", info);
        }
        println!();

        match state.take_action(&action) {
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
    let mut state = state::State::new(3).unwrap();

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
