use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WordGameError {
    NotYourTurn,
    GameAlreadyFinished,
    InvalidPlayerIndex,
    InvalidPlacement(String),
    InvalidWord(String),
    InsufficientTiles,
    ExchangeNotAllowed,
    TileNotInRack(char),
}

impl fmt::Display for WordGameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WordGameError::NotYourTurn => write!(f, "not your turn"),
            WordGameError::GameAlreadyFinished => write!(f, "game is already finished"),
            WordGameError::InvalidPlayerIndex => write!(f, "invalid player index"),
            WordGameError::InvalidPlacement(msg) => write!(f, "invalid placement: {msg}"),
            WordGameError::InvalidWord(word) => write!(f, "invalid word: {word}"),
            WordGameError::InsufficientTiles => {
                write!(f, "not enough tiles in bag for exchange")
            }
            WordGameError::ExchangeNotAllowed => {
                write!(f, "exchange not allowed when bag has fewer than 7 tiles")
            }
            WordGameError::TileNotInRack(ch) => write!(f, "tile '{ch}' not in rack"),
        }
    }
}

impl std::error::Error for WordGameError {}
