pub mod error;
pub mod game;
pub mod players;

use game::card::{Card, CardSuit, CardValue, Hand};
use game::enemy::Enemy;
use game::player::PlayerId;
use game::state::State;
use game::{Action, GameResult, GameStatus};
use players::{
    input_player::InputPlayer, mcts_player::MCTSPlayer, random_player::RandomPlayer, Play,
};
use pyo3::exceptions::{PyKeyError, PyTypeError, PyValueError};
use pyo3::types::{IntoPyDict, PyTuple};
use pyo3::{prelude::*, AsPyPointer};

/// A macro for a match statement that calls the same function
/// with the inner state for each variant of StateEnum
macro_rules! state_enum_repeat {
    ($to_match:expr, $to_repeat:expr $(, $additional_args:expr)*) => {
        match $to_match {
            StateEnum::Players1(state) => $to_repeat(state $(, $additional_args)*),
            StateEnum::Players2(state) => $to_repeat(state $(, $additional_args)*),
            StateEnum::Players3(state) => $to_repeat(state $(, $additional_args)*),
            StateEnum::Players4(state) => $to_repeat(state $(, $additional_args)*),
        }
    };
}

#[pyclass]
#[pyo3(name = "GameResult")]
#[derive(Debug)]
enum PyGameResult {
    Won,
    Lost,
}

#[pyclass]
struct RegicideGame {
    state: PyState,
    players: Vec<PyPlayer>,
}

#[pymethods]
impl RegicideGame {
    #[new]
    fn new(players: Vec<PyPlayer>, seed: Option<u64>) -> PyResult<Self> {
        let state = PyState {
            state_enum: StateEnum::new(&players, seed)?,
        };
        Ok(Self { state, players })
    }

    fn print(&self) {
        dbg!(&self.state);
    }

    fn playout(&mut self, py: Python) -> PyResult<PyGameResult> {
        loop {
            let state_enum_clone = self.state.state_enum;
            let player_id = self.state.has_turn();
            let player = self.players.get_mut(player_id.unwrap().0).unwrap();
            let py_state = self.state.clone().into_py(py);

            let action = match player {
                PyPlayer::Rust(rust_player) => rust_player.play(state_enum_clone),
                PyPlayer::Python(python_obj) => {
                    // let args = PyTuple::new(py, &["$self"]);
                    let args = PyTuple::empty(py);
                    let kwargs = vec![("state", py_state)].into_py_dict(py);
                    python_obj
                        // .call_method(py, "play", args, Some(kwargs))?
                        .call_method(py, "play", args, Some(kwargs))?
                        .extract::<PyAction>(py)?
                        .into()
                }
            };

            // Validate that the chosen action is legal
            let action_space = state_enum_repeat!(&self.state.state_enum, State::get_action_space);
            if !action_space.contains(&action) {
                return Err(PyKeyError::new_err(format!(
                    "'{:?}' is not a legal action. Legal actions are: {:?}",
                    action, action_space
                )));
            }

            if let Some(result) = self.state.state_enum.take_action(&action) {
                return Ok(result);
            }
        }
    }

    fn reward(&self) -> usize {
        self.state.reward().into()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StateEnum {
    Players1(State<1>),
    Players2(State<2>),
    Players3(State<3>),
    Players4(State<4>),
}

impl StateEnum {
    fn new(players: &Vec<PyPlayer>, seed: Option<u64>) -> Result<Self, PyErr> {
        match players.len() {
            1 => Ok(StateEnum::Players1(State::new(seed).unwrap())),
            2 => Ok(StateEnum::Players2(State::new(seed).unwrap())),
            3 => Ok(StateEnum::Players3(State::new(seed).unwrap())),
            4 => Ok(StateEnum::Players4(State::new(seed).unwrap())),
            _ => Err(PyValueError::new_err("Only 1-4 players are allowed")),
        }
    }

    fn take_action_generic<const N: usize>(
        state: &mut State<N>,
        action: &Action,
    ) -> Option<PyGameResult> {
        match state.take_action(action) {
            GameStatus::InProgress(new_state) => {
                *state = new_state;
                None
            }
            GameStatus::HasEnded(result) => match result {
                GameResult::Won => Some(PyGameResult::Won),
                GameResult::Lost(_) => Some(PyGameResult::Lost),
            },
        }
    }

    fn take_action(&mut self, action: &Action) -> Option<PyGameResult> {
        state_enum_repeat!(self, StateEnum::take_action_generic, action)
    }
}

#[derive(Clone, Debug)]
#[pyclass]
pub struct PyState {
    state_enum: StateEnum,
}

#[pymethods]
impl PyState {
    fn action_space(&self) -> Vec<PyAction> {
        state_enum_repeat!(&self.state_enum, State::get_action_space)
            .iter()
            .map(|&a| a.into())
            .collect()
    }

    fn has_turn(&self) -> PyResult<PlayerId> {
        Ok(state_enum_repeat!(&self.state_enum, State::has_turn))
    }

    fn current_hand(&self) -> Vec<Card> {
        state_enum_repeat!(&self.state_enum, State::current_hand)
            .into_iter()
            .collect()
    }

    fn reward(&self) -> u8 {
        state_enum_repeat!(&self.state_enum, State::reward)
    }

    fn current_enemy(&self) -> PyResult<Option<Enemy>> {
        Ok(state_enum_repeat!(&self.state_enum, State::current_enemy).copied())
    }

    fn __str__(&self) -> String {
        fn state_to_string<const N: usize>(state: &State<N>) -> String {
            format!("{}", state)
        }
        state_enum_repeat!(&self.state_enum, state_to_string)
    }
}

#[derive(Clone, FromPyObject)]
enum RustPlayer {
    RandomPlayer(RandomPlayer),
    InputPlayer(InputPlayer),
    MCTSPlayer(MCTSPlayer),
}

#[derive(Clone, FromPyObject)]
enum PyPlayer {
    Rust(RustPlayer),
    Python(PyObject),
}

impl RustPlayer {
    fn play_generic<const N: usize>(&mut self, state: State<N>) -> Action {
        match self {
            RustPlayer::RandomPlayer(player) => Play::play(player, state),
            RustPlayer::InputPlayer(player) => Play::play(player, state),
            RustPlayer::MCTSPlayer(player) => Play::play(player, state),
        }
    }

    fn play(&mut self, state_enum: StateEnum) -> Action {
        state_enum_repeat!(state_enum, |state| self.play_generic(state))
    }
}

impl AsPyPointer for RustPlayer {
    fn as_ptr(&self) -> *mut pyo3::ffi::PyObject {
        todo!()
    }
}

impl IntoPy<PyObject> for PyPlayer {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            PyPlayer::Rust(inner) => inner.into_py(py),
            PyPlayer::Python(inner) => inner.into_py(py),
        }
    }
}

#[derive(Clone, FromPyObject)]
pub enum PyAction {
    PyActionPlay(PyActionPlay),
    PyActionAnimalCombo(PyActionAnimalCombo),
    PyActionCombo(PyActionCombo),
    PyActionYield(PyActionYield),
    PyActionDiscard(PyActionDiscard),
    PyActionChangePlayer(PyActionChangePlayer),
    PyActionRefillHand(PyActionRefillHand),
}

impl Into<Action> for PyAction {
    fn into(self) -> Action {
        match self {
            PyAction::PyActionPlay(PyActionPlay(card)) => Action::Play(card),
            PyAction::PyActionAnimalCombo(PyActionAnimalCombo(c1, c2)) => {
                Action::AnimalCombo(c1, c2)
            }
            PyAction::PyActionYield(_) => Action::Yield,
            PyAction::PyActionCombo(PyActionCombo(cards)) => Action::Combo(cards),
            PyAction::PyActionDiscard(PyActionDiscard(hand)) => Action::Discard(hand),
            PyAction::PyActionChangePlayer(PyActionChangePlayer(id)) => Action::ChangePlayer(id),
            PyAction::PyActionRefillHand(PyActionRefillHand) => Action::RefillHand,
        }
    }
}

impl From<Action> for PyAction {
    fn from(action: Action) -> Self {
        match action {
            Action::Play(card) => PyAction::PyActionPlay(PyActionPlay(card)),
            Action::AnimalCombo(c1, c2) => {
                PyAction::PyActionAnimalCombo(PyActionAnimalCombo(c1, c2))
            }
            Action::Yield => PyAction::PyActionYield(PyActionYield),
            Action::Combo(cards) => PyAction::PyActionCombo(PyActionCombo(cards)),
            Action::Discard(hand) => PyAction::PyActionDiscard(PyActionDiscard(hand)),
            Action::ChangePlayer(id) => PyAction::PyActionChangePlayer(PyActionChangePlayer(id)),
            Action::RefillHand => PyAction::PyActionRefillHand(PyActionRefillHand),
        }
    }
}

impl IntoPy<PyObject> for PyAction {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            PyAction::PyActionPlay(inner) => inner.into_py(py),
            PyAction::PyActionAnimalCombo(inner) => inner.into_py(py),
            PyAction::PyActionYield(inner) => inner.into_py(py),
            PyAction::PyActionCombo(inner) => inner.into_py(py),
            PyAction::PyActionDiscard(inner) => inner.into_py(py),
            PyAction::PyActionChangePlayer(inner) => inner.into_py(py),
            PyAction::PyActionRefillHand(inner) => inner.into_py(py),
        }
    }
}

/// Boilerplate implementation for a PyAction
macro_rules! define_py_action {
    (
        $(#[$struct_meta:meta])*
        struct $action_name:ident $(( $($field_type:ty),* ))?,
        $(
            $(#[$fn_meta:meta])*
            fn $fn_name:ident($($arg:ident : $arg_type:ty),*) $(-> $fn_return_type:ty)?
                $fn_body:block
        )*
    ) => {
        #[derive(Clone, PartialEq)]
        #[pyclass]
        $(#[$struct_meta])*
        pub struct $action_name $(( $($field_type),* ))?;

        #[pymethods]
        impl $action_name {
            $(
                $(#[$fn_meta])*
                fn $fn_name($($arg: $arg_type),*) $(-> $fn_return_type)? {
                    $fn_body
                }
            )*

            fn __str__(&self) -> String {
                let action: Action = PyAction::$action_name(self.clone()).into();
                format!("{:?}", action)
            }

            fn __repr__(&self) -> String {
                let action: Action = PyAction::$action_name(self.clone()).into();
                format!("{:?}", action)
            }

            fn __richcmp__(&self, other: &Self, op: pyo3::basic::CompareOp) -> PyResult<bool> {
                match op {
                    pyo3::pyclass::CompareOp::Eq => Ok(self == other),
                    pyo3::pyclass::CompareOp::Ne => Ok(self != other),
                    _ => Err(PyTypeError::new_err("Operation not supported")),
                }
            }
        }
    }
}

define_py_action!(
    #[pyo3(name = "ActionPlay")]
    struct PyActionPlay(Card),
    #[new]
    fn new(card: Card) -> Self {
        Self(card)
    }
);

define_py_action!(
    #[pyo3(name = "ActionAnimalCombo")]
    struct PyActionAnimalCombo(Card, Card),
    #[new]
    fn new(c1: Card, c2: Card) -> Self {
        Self(c1, c2)
    }
);

define_py_action!(
    #[pyo3(name = "ActionCombo")]
    struct PyActionCombo(arrayvec::ArrayVecCopy<Card, 4>),
    #[new]
    fn new(cards: Vec<Card>) -> Self {
        let cards_arr = arrayvec::ArrayVecCopy::from_iter(cards.into_iter());
        Self(cards_arr)
    }
);

define_py_action!(
    #[pyo3(name = "ActionYield")]
    struct PyActionYield,
    #[new]
    fn new() -> Self {
        Self
    }
);

define_py_action!(
    #[pyo3(name = "ActionDiscard")]
    struct PyActionDiscard(Hand),
    #[new]
    fn new(cards: Vec<Card>) -> Self {
        let hand = Hand::from_iter(cards.into_iter());
        Self(hand)
    }
);

define_py_action!(
    #[pyo3(name = "ActionChangePlayer")]
    struct PyActionChangePlayer(PlayerId),
    #[new]
    fn new(id: usize) -> Self {
        Self(PlayerId(id))
    }
);

define_py_action!(
    #[pyo3(name = "ActionRefillHand")]
    struct PyActionRefillHand,
    #[new]
    fn new() -> Self {
        Self
    }
);

/// A Python module implemented in Rust.
#[pymodule]
fn regicide(py: Python, m: &PyModule) -> PyResult<()> {
    let card = PyModule::new(py, "card")?;
    card.add_class::<Card>()?;
    card.add_class::<CardSuit>()?;
    card.add_class::<CardValue>()?;
    m.add_submodule(card)?;

    let actions = PyModule::new(py, "actions")?;
    actions.add_class::<PyActionPlay>()?;
    actions.add_class::<PyActionAnimalCombo>()?;
    actions.add_class::<PyActionCombo>()?;
    actions.add_class::<PyActionYield>()?;
    actions.add_class::<PyActionDiscard>()?;
    actions.add_class::<PyActionChangePlayer>()?;
    m.add_submodule(actions)?;

    let players = PyModule::new(py, "players")?;
    players.add_class::<RandomPlayer>()?;
    players.add_class::<InputPlayer>()?;
    players.add_class::<MCTSPlayer>()?;
    m.add_submodule(players)?;

    m.add_class::<RegicideGame>()?;

    Ok(())
}
