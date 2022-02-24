// use std::convert::TryInto;
use std::io::stdin;

// use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::thread_rng;
use regicide_rl::game::state;
use regicide_rl::game::{GameResult, GameStatus};

fn main() {
    // let n_tries = 100;
    // let wins = (0..n_tries)
    //     .map(|_| random_playout())
    //     .filter(|r| match r {
    //         GameResult::Won => true,
    //         _ => false,
    //     })
    //     .count();
    // println!("{}/{} wins", wins, n_tries);
    let result = input_playout();
    dbg!(result);
}

fn random_playout() -> GameResult {
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

fn input_playout() -> GameResult {
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
