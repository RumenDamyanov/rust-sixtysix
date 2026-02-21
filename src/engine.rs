//! Core engine: Game trait, Action, Session, Store trait, and Engine orchestrator.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};

// ---------------------------------------------------------------------------
// Action
// ---------------------------------------------------------------------------

/// A generic instruction sent by a client/actor.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    #[serde(rename = "type")]
    pub action_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<HashMap<String, serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

// ---------------------------------------------------------------------------
// Game trait
// ---------------------------------------------------------------------------

/// Defines the logic for a particular game.
///
/// Implementations must be deterministic and pure: given the same input state
/// and action they must return the same output state.
pub trait Game: Send + Sync {
    /// A stable, unique name for the game (e.g. "sixtysix").
    fn name(&self) -> &str;

    /// Create the starting state. `seed` enables deterministic randomness.
    fn initial_state(&self, seed: i64) -> serde_json::Value;

    /// Check whether an action is valid given the state. Must not mutate state.
    fn validate(&self, state: &serde_json::Value, action: &Action) -> Result<(), GameError>;

    /// Return a new state after applying the action. Must not mutate input.
    fn apply(
        &self,
        state: &serde_json::Value,
        action: &Action,
    ) -> Result<serde_json::Value, GameError>;
}

// ---------------------------------------------------------------------------
// Session
// ---------------------------------------------------------------------------

/// A single instance of a game.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub game_name: String,
    pub state: serde_json::Value,
    pub version: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Store trait
// ---------------------------------------------------------------------------

/// Abstracts persistence for sessions.
pub trait Store: Send + Sync {
    fn create(&self, session: Session) -> Result<(), EngineError>;
    fn get(&self, id: &str) -> Result<Option<Session>, EngineError>;
    fn update(&self, session: Session) -> Result<(), EngineError>;
    fn list(
        &self,
        game_name: &str,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<Session>, EngineError>;
    fn delete(&self, id: &str) -> Result<(), EngineError>;
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors produced by the game logic.
#[derive(Debug, Clone)]
pub struct GameError(pub String);

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for GameError {}

/// Errors produced by the engine / store layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineError {
    GameNotFound,
    SessionNotFound,
    Conflict,
    Validation(String),
    Store(String),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GameNotFound => write!(f, "engine: game not found"),
            Self::SessionNotFound => write!(f, "engine: session not found"),
            Self::Conflict => write!(f, "engine: conflict"),
            Self::Validation(msg) => write!(f, "{msg}"),
            Self::Store(msg) => write!(f, "store: {msg}"),
        }
    }
}

impl std::error::Error for EngineError {}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// Wires games with storage and provides a simple API to manipulate sessions.
pub struct Engine {
    store: Arc<dyn Store>,
    games: RwLock<HashMap<String, Arc<dyn Game>>>,
}

impl Engine {
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self {
            store,
            games: RwLock::new(HashMap::new()),
        }
    }

    /// Register a game. Panics if a game with the same name already exists.
    pub fn register(&self, game: Arc<dyn Game>) {
        let name = game.name().to_string();
        assert!(!name.is_empty(), "engine: game must have a non-empty name");
        let mut games = self.games.write().unwrap();
        assert!(
            !games.contains_key(&name),
            "engine: duplicate game name: {name}"
        );
        games.insert(name, game);
    }

    /// Returns registered game names.
    pub fn games(&self) -> Vec<String> {
        let games = self.games.read().unwrap();
        games.keys().cloned().collect()
    }

    /// Create a new session for the named game.
    pub fn create_session(&self, game_name: &str, seed: i64) -> Result<Session, EngineError> {
        let games = self.games.read().unwrap();
        let game = games.get(game_name).ok_or(EngineError::GameNotFound)?;
        let state = game.initial_state(seed);
        let now = Utc::now();
        let session = Session {
            id: random_id(),
            game_name: game_name.to_string(),
            state,
            version: 1,
            created_at: now,
            updated_at: now,
        };
        self.store.create(session.clone())?;
        Ok(session)
    }

    /// Get a session by id.
    pub fn get_session(&self, id: &str) -> Result<Session, EngineError> {
        self.store.get(id)?.ok_or(EngineError::SessionNotFound)
    }

    /// Validate and apply an action to the session state.
    pub fn apply_action(&self, id: &str, action: Action) -> Result<Session, EngineError> {
        let mut session = self.store.get(id)?.ok_or(EngineError::SessionNotFound)?;

        let games = self.games.read().unwrap();
        let game = games
            .get(&session.game_name)
            .ok_or(EngineError::GameNotFound)?;

        game.validate(&session.state, &action)
            .map_err(|e| EngineError::Validation(e.0))?;

        let new_state = game
            .apply(&session.state, &action)
            .map_err(|e| EngineError::Validation(e.0))?;

        session.state = new_state;
        session.version += 1;
        session.updated_at = Utc::now();
        self.store.update(session.clone())?;
        Ok(session)
    }

    /// List sessions for a given game.
    pub fn list_sessions(
        &self,
        game_name: &str,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<Session>, EngineError> {
        self.store.list(game_name, offset, limit)
    }

    /// Delete a session.
    pub fn delete_session(&self, id: &str) -> Result<(), EngineError> {
        self.store.delete(id)
    }
}

/// Generate a random hex ID (32 hex chars = 16 bytes).
fn random_id() -> String {
    let mut buf = [0u8; 16];
    // Use getrandom via rand
    use rand::RngCore;
    rand::thread_rng().fill_bytes(&mut buf);
    hex::encode(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::SixtySix;
    use crate::store::Memory;

    #[test]
    fn engine_create_get_apply_list_delete() {
        let mem = Arc::new(Memory::new());
        let engine = Engine::new(mem);
        engine.register(Arc::new(SixtySix));

        // create session
        let s = engine.create_session("sixtysix", 0).unwrap();
        assert_eq!(s.game_name, "sixtysix");
        assert_eq!(s.version, 1);

        // get session
        let gs = engine.get_session(&s.id).unwrap();
        assert_eq!(gs.id, s.id);

        // apply closeStock action
        let action = Action {
            action_type: "closeStock".to_string(),
            actor: None,
            payload: None,
            idempotency_key: None,
        };
        engine.apply_action(&s.id, action).unwrap();

        // list
        let list = engine.list_sessions("sixtysix", 0, 10).unwrap();
        assert!(!list.is_empty());

        // delete
        engine.delete_session(&s.id).unwrap();
    }
}
