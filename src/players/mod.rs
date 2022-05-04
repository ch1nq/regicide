use crate::{
    game::{state::State, Action},
    PyAction, PyState, StateEnum,
};

pub trait Play {
    fn play<const N: usize>(&mut self, state: State<N>) -> Action;

    fn play_py(&mut self, state: PyState) -> PyAction {
        match state.state_enum {
            StateEnum::Players1(state) => self.play(state),
            StateEnum::Players2(state) => self.play(state),
            StateEnum::Players3(state) => self.play(state),
            StateEnum::Players4(state) => self.play(state),
        }
        .into()
    }
}

pub mod input_player;
pub mod mcts_player;
pub mod random_player;
