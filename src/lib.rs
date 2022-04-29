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
use pyo3::exceptions::{PyKeyError, PyValueError};
use pyo3::types::{IntoPyDict, PyTuple};
use pyo3::{prelude::*, AsPyPointer};

#[pyclass]
#[pyo3(name = "GameResult")]
enum PyGameResult {
    Won,
    Lost,
}

#[pyclass]
struct RegicideGame {
    state: StateEnum,
    players: Vec<PyPlayer>,
}

impl RegicideGame {
    fn generic_playout<const N: usize>(
        mut players: Vec<PyPlayer>,
        mut state: State<N>,
        py: Python,
    ) -> PyResult<PyGameResult> {
        loop {
            if let Some(e) = state.current_enemy() {
                println!("{}: {:?}", state.reward(), e);
            }

            let player_id = state.has_turn();
            let player = players.get_mut(player_id.0).unwrap();
            let py_state = PyState::from_state(state).into_py(py);

            let action = match player {
                PyPlayer::Rust(rust_player) => match rust_player {
                    RustPlayer::RandomPlayer(p) => p.play(state),
                    RustPlayer::InputPlayer(p) => p.play(state),
                    RustPlayer::MCTSPlayer(p) => p.play(state),
                },
                PyPlayer::Python(python_obj) => {
                    let args = PyTuple::new(py, &["$self"]);
                    let kwargs = vec![("state", py_state)].into_py_dict(py);
                    python_obj
                        .call_method(py, "play", args, Some(kwargs))?
                        .extract::<PyAction>(py)?
                        .into()
                }
            };

            println!("{:?}\n", action);

            // Validate that the chosen action is legal
            let action_space = state.get_action_space();
            if !action_space.contains(&action) {
                return Err(PyKeyError::new_err(format!(
                    "Action '{:?}' is not a legal move. Legal moves are: {:?}",
                    action, action_space
                )));
            }

            match state.take_action(&action) {
                GameStatus::InProgress(new_state) => {
                    state = new_state;
                }
                GameStatus::HasEnded(result) => {
                    return match result {
                        GameResult::Won => Ok(PyGameResult::Won),
                        GameResult::Lost(_) => Ok(PyGameResult::Lost),
                    };
                }
            };
        }
    }
}

#[pymethods]
impl RegicideGame {
    #[new]
    fn new(players: Vec<PyPlayer>, seed: Option<u64>) -> PyResult<Self> {
        let state = StateEnum::new(&players, seed)?;
        Ok(Self { state, players })
    }

    fn print(&self) {
        dbg!(&self.state);
    }

    fn playout(&self, py: Python) -> PyResult<PyGameResult> {
        let players = self.players.clone();
        match self.state {
            StateEnum::Players1(s) => Self::generic_playout(players, s, py),
            StateEnum::Players2(s) => Self::generic_playout(players, s, py),
            StateEnum::Players3(s) => Self::generic_playout(players, s, py),
            StateEnum::Players4(s) => Self::generic_playout(players, s, py),
        }
    }
}

#[derive(Debug, Clone)]
enum StateEnum {
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

    fn action_space(&self) -> Vec<Action> {
        match self {
            StateEnum::Players1(state) => state.get_action_space(),
            StateEnum::Players2(state) => state.get_action_space(),
            StateEnum::Players3(state) => state.get_action_space(),
            StateEnum::Players4(state) => state.get_action_space(),
        }
    }

    fn current_enemy(&self) -> Option<Enemy> {
        match self {
            StateEnum::Players1(state) => state.current_enemy(),
            StateEnum::Players2(state) => state.current_enemy(),
            StateEnum::Players3(state) => state.current_enemy(),
            StateEnum::Players4(state) => state.current_enemy(),
        }
        .copied()
    }
}

#[derive(Clone)]
#[pyclass]
struct PyState {
    // state_enum: StateEnum,
}

impl PyState {
    fn from_state<const N: usize>(state: State<N>) -> Self {
        Self {}
        // Self {
        //     state_enum: match N {
        //         1 => StateEnum::Players1(state.into()),
        //         _ => todo!(),
        //     },
        // }
    }
}

// #[pymethods]
// impl PyState {
//     fn actions_space(&self) -> Vec<PyAction> {
//         self.state_enum
//             .action_space()
//             .iter()
//             .map(|&a| a.into())
//             .collect()
//     }

//     fn current_enemy(&self) -> PyResult<Option<Enemy>> {
//         Ok(self.state_enum.current_enemy())
//     }
// }

#[derive(Clone, FromPyObject)]
enum RustPlayer {
    RandomPlayer(RandomPlayer),
    InputPlayer(InputPlayer),
    MCTSPlayer(MCTSPlayer),
}

impl AsPyPointer for RustPlayer {
    fn as_ptr(&self) -> *mut pyo3::ffi::PyObject {
        todo!()
    }
}

#[derive(Clone, FromPyObject)]
enum PyPlayer {
    Rust(RustPlayer),
    Python(PyObject),
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
enum PyAction {
    PyActionPlay(PyActionPlay),
    PyActionAnimalCombo(PyActionAnimalCombo),
    PyActionCombo(PyActionCombo),
    PyActionYield(PyActionYield),
    PyActionDiscard(PyActionDiscard),
    PyActionChangePlayer(PyActionChangePlayer),
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
        }
    }
}

#[derive(Clone)]
#[pyclass]
#[pyo3(name = "ActionPlay")]
struct PyActionPlay(Card);

#[derive(Clone)]
#[pyclass]
#[pyo3(name = "ActionAnimalCombo")]
struct PyActionAnimalCombo(Card, Card);

#[derive(Clone)]
#[pyclass]
#[pyo3(name = "ActionCombo")]
struct PyActionCombo(arrayvec::ArrayVecCopy<Card, 4>);

#[derive(Clone)]
#[pyclass]
#[pyo3(name = "ActionYield")]
struct PyActionYield;

#[derive(Clone)]
#[pyclass]
#[pyo3(name = "ActionDiscard")]
struct PyActionDiscard(Hand);

#[derive(Clone)]
#[pyclass]
#[pyo3(name = "ActionChangePlayer")]
struct PyActionChangePlayer(PlayerId);

#[pymethods]
impl PyActionPlay {
    #[new]
    fn new(card: Card) -> Self {
        Self(card)
    }
}
#[pymethods]
impl PyActionAnimalCombo {
    #[new]
    fn new(c1: Card, c2: Card) -> Self {
        Self(c1, c2)
    }
}
#[pymethods]
impl PyActionCombo {
    #[new]
    fn new(cards: Vec<Card>) -> Self {
        let cards_arr = arrayvec::ArrayVecCopy::from_iter(cards.into_iter());
        Self(cards_arr)
    }
}
#[pymethods]
impl PyActionYield {
    #[new]
    fn new() -> Self {
        Self
    }
}
#[pymethods]
impl PyActionChangePlayer {
    #[new]
    fn new(id: usize) -> Self {
        Self(PlayerId(id))
    }
}

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
