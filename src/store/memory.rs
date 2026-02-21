//! Thread-safe in-memory store useful for tests and small deployments.

use crate::engine::{EngineError, Session, Store};
use std::collections::HashMap;
use std::sync::RwLock;

pub struct Memory {
    sessions: RwLock<HashMap<String, Session>>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl Store for Memory {
    fn create(&self, session: Session) -> Result<(), EngineError> {
        let mut map = self.sessions.write().unwrap();
        if map.contains_key(&session.id) {
            return Err(EngineError::Store("duplicate id".to_string()));
        }
        map.insert(session.id.clone(), session);
        Ok(())
    }

    fn get(&self, id: &str) -> Result<Option<Session>, EngineError> {
        let map = self.sessions.read().unwrap();
        Ok(map.get(id).cloned())
    }

    fn update(&self, session: Session) -> Result<(), EngineError> {
        let mut map = self.sessions.write().unwrap();
        if !map.contains_key(&session.id) {
            return Err(EngineError::SessionNotFound);
        }
        map.insert(session.id.clone(), session);
        Ok(())
    }

    fn list(
        &self,
        game_name: &str,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<Session>, EngineError> {
        let map = self.sessions.read().unwrap();
        let mut all: Vec<Session> = map
            .values()
            .filter(|s| game_name.is_empty() || s.game_name == game_name)
            .cloned()
            .collect();
        all.sort_by_key(|s| s.created_at);

        if offset >= all.len() {
            return Ok(vec![]);
        }
        let end = if limit == 0 {
            all.len()
        } else {
            (offset + limit).min(all.len())
        };
        Ok(all[offset..end].to_vec())
    }

    fn delete(&self, id: &str) -> Result<(), EngineError> {
        let mut map = self.sessions.write().unwrap();
        if map.remove(id).is_none() {
            return Err(EngineError::SessionNotFound);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn memory_crud() {
        let m = Memory::new();
        let now = Utc::now();
        let s = Session {
            id: "a".to_string(),
            game_name: "g".to_string(),
            state: serde_json::json!({}),
            version: 1,
            created_at: now,
            updated_at: now,
        };

        // create
        m.create(s.clone()).unwrap();

        // duplicate
        assert!(m.create(s.clone()).is_err());

        // get
        let got = m.get("a").unwrap().unwrap();
        assert_eq!(got.id, "a");

        // update
        let mut updated = got;
        updated.version = 2;
        m.update(updated).unwrap();

        // list
        let list = m.list("g", 0, 10).unwrap();
        assert_eq!(list.len(), 1);

        // delete
        m.delete("a").unwrap();

        // delete non-existent
        assert!(m.delete("a").is_err());
    }
}
