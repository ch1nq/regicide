use std::error::Error;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum RegicideError {
    WrongNumberOfPlayers,
}

impl std::fmt::Display for RegicideError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?})", self))
    }
}

impl Error for RegicideError {}
