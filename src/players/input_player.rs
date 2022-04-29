use crate::{game::state::State, Action};
use pyo3::prelude::*;
use std::io::stdin;

use super::Play;

#[derive(Clone)]
#[pyclass]
pub struct InputPlayer;

#[pymethods]
impl InputPlayer {
    #[new]
    fn new() -> Self {
        Self
    }
}

impl Play for InputPlayer {
    fn play<const N: usize>(&mut self, state: State<N>) -> Action {
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
        println!("Type number to select an action:");

        loop {
            let mut input = String::new();

            stdin().read_line(&mut input).expect("Error reading input.");
            if let Ok(idx) = input.trim().parse::<usize>() {
                if let Some(&action) = actions.get(idx) {
                    return action;
                } else {
                    println!("Could not match input with any action. Please try again!");
                }
            } else {
                println!("Failed to parse input {:?}. Please try again!", input);
            }
        }
    }
}
