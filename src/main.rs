mod card;
mod game;
mod player;
mod state;
mod table;

use rand::seq::SliceRandom;
use rand::thread_rng;

fn main() {
    let state = state::State::new(2).unwrap();
    // dbg!(&state);

    for _ in 0..10 {
        let mut rng = thread_rng();
        let actions = state.get_action_space();
        dbg!(&state.players[0].hand);
        dbg!(actions);
        return;
        let action = actions.choose(&mut rng).unwrap();
        dbg!(action);
        // let state = state.take_action(action);
    }
}
