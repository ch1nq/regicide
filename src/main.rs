use rand::seq::SliceRandom;
use rand::thread_rng;
use regicide_rl::game::state;
use regicide_rl::game::{GameResult, GameStatus};

fn main() {
    let n_tries = 100;
    let wins = (0..n_tries)
        .map(|_| random_playout())
        .filter(|r| match r {
            GameResult::Won => true,
            _ => false,
        })
        .count();
    println!("{}/{} wins", wins, n_tries);
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
