use rand::prelude::SliceRandom;
use rand::thread_rng;

#[allow(dead_code)]
mod logic;

fn main() {
    let state = logic::State::new(2).unwrap();
    // dbg!(&state);
    for _ in 0..10 {
        let mut rng = thread_rng();
        let actions = state.get_action_space();
        let action = actions.choose(&mut rng).unwrap();
        dbg!(action);
        let state = state.take_action(action);
    }
}
