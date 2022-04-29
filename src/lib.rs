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
                    let args = PyTuple::new(py, &["$self"]);
                    let kwargs = vec![("state", py_state)].into_py_dict(py);
                    python_obj
                        .call_method(py, "play", args, Some(kwargs))?
                        .extract::<PyAction>(py)?
                        .into()
                }
            };

            // Validate that the chosen action is legal
            let action_space = self.state.action_space();
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
}

#[derive(Debug, Clone, Copy)]
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

    fn take_action_generic<const N: usize>(
        x: GameStatus<N>,
        state: &mut State<N>,
    ) -> Option<PyGameResult> {
        match x {
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
        match self {
            StateEnum::Players1(s) => Self::take_action_generic(s.take_action(action), s),
            StateEnum::Players2(s) => Self::take_action_generic(s.take_action(action), s),
            StateEnum::Players3(s) => Self::take_action_generic(s.take_action(action), s),
            StateEnum::Players4(s) => Self::take_action_generic(s.take_action(action), s),
        }
    }
}

#[derive(Clone, Debug)]
#[pyclass]
struct PyState {
    state_enum: StateEnum,
}

impl PyState {
    fn action_space(&self) -> Vec<Action> {
        match self.state_enum {
            StateEnum::Players1(state) => state.get_action_space(),
            StateEnum::Players2(state) => state.get_action_space(),
            StateEnum::Players3(state) => state.get_action_space(),
            StateEnum::Players4(state) => state.get_action_space(),
        }
    }
}

#[pymethods]
impl PyState {
    #[pyo3(name = "action_space")]
    fn py_action_space(&self) -> Vec<PyAction> {
        self.action_space().iter().map(|&a| a.into()).collect()
    }

    fn has_turn(&self) -> PyResult<PlayerId> {
        let id = match self.state_enum {
            StateEnum::Players1(state) => state.has_turn(),
            StateEnum::Players2(state) => state.has_turn(),
            StateEnum::Players3(state) => state.has_turn(),
            StateEnum::Players4(state) => state.has_turn(),
        };
        Ok(id)
    }

    fn reward(&self) -> u8 {
        match self.state_enum {
            StateEnum::Players1(state) => state.reward(),
            StateEnum::Players2(state) => state.reward(),
            StateEnum::Players3(state) => state.reward(),
            StateEnum::Players4(state) => state.reward(),
        }
    }

    fn current_enemy(&self) -> PyResult<Option<Enemy>> {
        let enemy = match self.state_enum {
            StateEnum::Players1(state) => state.current_enemy().copied(),
            StateEnum::Players2(state) => state.current_enemy().copied(),
            StateEnum::Players3(state) => state.current_enemy().copied(),
            StateEnum::Players4(state) => state.current_enemy().copied(),
        };
        Ok(enemy)
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
            RustPlayer::RandomPlayer(player) => player.play(state),
            RustPlayer::InputPlayer(player) => player.play(state),
            RustPlayer::MCTSPlayer(player) => player.play(state),
        }
    }

    fn play(&mut self, state_enum: StateEnum) -> Action {
        match state_enum {
            StateEnum::Players1(state) => self.play_generic(state),
            StateEnum::Players2(state) => self.play_generic(state),
            StateEnum::Players3(state) => self.play_generic(state),
            StateEnum::Players4(state) => self.play_generic(state),
        }
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

    fn __str__(&self) -> String {
        let action: Action = PyAction::PyActionPlay(self.clone()).into();
        format!("{:?}", action)
    }
}
#[pymethods]
impl PyActionAnimalCombo {
    #[new]
    fn new(c1: Card, c2: Card) -> Self {
        Self(c1, c2)
    }

    fn __str__(&self) -> String {
        let action: Action = PyAction::PyActionAnimalCombo(self.clone()).into();
        format!("{:?}", action)
    }
}
#[pymethods]
impl PyActionCombo {
    #[new]
    fn new(cards: Vec<Card>) -> Self {
        let cards_arr = arrayvec::ArrayVecCopy::from_iter(cards.into_iter());
        Self(cards_arr)
    }

    fn __str__(&self) -> String {
        let action: Action = PyAction::PyActionCombo(self.clone()).into();
        format!("{:?}", action)
    }
}
#[pymethods]
impl PyActionYield {
    #[new]
    fn new() -> Self {
        Self
    }

    fn __str__(&self) -> String {
        let action: Action = PyAction::PyActionYield(self.clone()).into();
        format!("{:?}", action)
    }
}
#[pymethods]
impl PyActionDiscard {
    #[new]
    fn new(cards: Vec<Card>) -> Self {
        let hand = Hand::from_iter(cards.into_iter());
        Self(hand)
    }

    fn __str__(&self) -> String {
        let action: Action = PyAction::PyActionDiscard(self.clone()).into();
        format!("{:?}", action)
    }
}
#[pymethods]
impl PyActionChangePlayer {
    #[new]
    fn new(id: usize) -> Self {
        Self(PlayerId(id))
    }

    fn __str__(&self) -> String {
        let action: Action = PyAction::PyActionChangePlayer(self.clone()).into();
        format!("{:?}", action)
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
