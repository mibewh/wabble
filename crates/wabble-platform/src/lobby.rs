use serde::{Deserialize, Serialize};

use crate::player::PlayerId;

/// A lobby where players gather before a match starts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lobby {
    pub id: u64,
    pub host: PlayerId,
    pub game_type: String,
    pub players: Vec<PlayerId>,
    pub max_players: usize,
    pub config_data: Vec<u8>,
    pub created_at: u64,
}

impl Lobby {
    pub fn new(
        id: u64,
        host: PlayerId,
        game_type: String,
        max_players: usize,
        config_data: Vec<u8>,
        now: u64,
    ) -> Self {
        Self {
            id,
            host,
            game_type,
            players: vec![host],
            max_players,
            config_data,
            created_at: now,
        }
    }

    pub fn is_full(&self) -> bool {
        self.players.len() >= self.max_players
    }

    pub fn join(&mut self, player: PlayerId) -> Result<(), LobbyError> {
        if self.is_full() {
            return Err(LobbyError::Full);
        }
        if self.players.contains(&player) {
            return Err(LobbyError::AlreadyJoined);
        }
        self.players.push(player);
        Ok(())
    }

    pub fn leave(&mut self, player: PlayerId) -> Result<(), LobbyError> {
        let pos = self
            .players
            .iter()
            .position(|p| *p == player)
            .ok_or(LobbyError::NotInLobby)?;
        self.players.remove(pos);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LobbyError {
    Full,
    AlreadyJoined,
    NotInLobby,
}

impl std::fmt::Display for LobbyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LobbyError::Full => write!(f, "lobby is full"),
            LobbyError::AlreadyJoined => write!(f, "player already in lobby"),
            LobbyError::NotInLobby => write!(f, "player not in lobby"),
        }
    }
}

impl std::error::Error for LobbyError {}
